# coding: utf-8
import time
import typing

from synergine2.simulation import SubjectBehaviour, SubjectMechanism
from synergine2.simulation import Event
from synergine2_xyz.move.intention import MoveToIntention
from synergine2_xyz.simulation import XYZSimulation
from synergine2_xyz.utils import get_angle

from opencombat.const import COLLECTION_ALIVE
from opencombat.user_action import UserAction


class SubjectStartRotationEvent(Event):
    def __init__(
        self,
        subject_id: int,
        rotate_relative: float,
        rotate_absolute: float,
        duration: float,
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.rotate_relative = rotate_relative
        self.rotate_absolute = rotate_absolute
        self.duration = duration
        self.gui_action = gui_action


class SubjectContinueRotationEvent(Event):
    def __init__(
        self,
        subject_id: int,
        rotate_relative: float,
        duration: float,
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.rotate_relative = rotate_relative
        self.duration = duration
        self.gui_action = gui_action


class SubjectFinishRotationEvent(Event):
    def __init__(
        self,
        subject_id: int,
        rotation_absolute: float,
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.rotation_absolute = rotation_absolute
        self.gui_action = gui_action


class SubjectStartTileMoveEvent(Event):
    def __init__(
        self,
        subject_id: int,
        move_to: typing.Tuple[int, int],
        duration: float,
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.move_to = move_to
        self.duration = duration
        self.gui_action = gui_action


class SubjectContinueTileMoveEvent(Event):
    def __init__(
        self,
        subject_id: int,
        move_to: typing.Tuple[int, int],
        duration: float,
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.move_to = move_to
        self.duration = duration
        self.gui_action = gui_action


class SubjectFinishTileMoveEvent(Event):
    def __init__(
        self,
        subject_id: int,
        move_to: typing.Tuple[int, int],
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.move_to = move_to
        self.gui_action = gui_action


class SubjectFinishMoveEvent(Event):
    def __init__(
        self,
        subject_id: int,
        move_to: typing.Tuple[int, int],
        gui_action: UserAction,
    ) -> None:
        self.subject_id = subject_id
        self.move_to = move_to
        self.gui_action = gui_action


class MoveToMechanism(SubjectMechanism):
    def run(self) -> dict:
        try:
            # TODO: MoveToIntention doit être configurable
            move = self.subject.intentions.get(MoveToIntention)  # type: MoveToIntention
        except KeyError:
            return {}

        if COLLECTION_ALIVE not in self.subject.collections:
            return {}

        return move.get_data()


class MoveWithRotationBehaviour(SubjectBehaviour):
    use = [MoveToMechanism]

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.simulation = typing.cast(XYZSimulation, self.simulation)

    def run(self, data) -> object:
        """
        Compute data relative to move
        """
        data = data[MoveToMechanism]
        if not data:
            return False

        # Prepare data
        to = data['to']  # type: typing.Tuple(int, int)
        return_data = {}
        now = time.time()

        # Test if it's first time
        if not data.get('path'):
            return_data['path'] = self.simulation.physics.found_path(
                start=self.subject.position,
                end=to,
                subject=self.subject,
            )
            # find path algorithm can skip start position, add it if not in
            if return_data['path'][0] != self.subject.position:
                return_data['path'] = [self.subject.position] + return_data['path']
            data['path'] = return_data['path']

        # Prepare data
        path = data['path']  # type: typing.List[typing.Tuple(int, int)]
        path_index = path.index(self.subject.position)
        next_position = path[path_index + 1]
        next_position_direction = get_angle(self.subject.position, next_position)
        rotate_relative = next_position_direction - self.subject.direction

        # Test if finish move
        if path_index == len(path) - 1:
            return {
                'move_to_finished': to,
                'gui_action': data['gui_action'],
            }

        # Check if moving
        if self.subject.moving_to == next_position:
            if self.subject.start_move + self.subject.move_duration > now:
                # Let moving
                return {
                    'tile_move_to': next_position,
                    'gui_action': data['gui_action'],
                }
            return_data['tile_move_to_finished'] = self.subject.moving_to
            # Must consider new position of subject
            path_index = path.index(return_data['tile_move_to_finished'])
            if path_index == len(path) - 1:
                return {
                    'move_to_finished': to,
                    'gui_action': data['gui_action'],
                }
            next_position = path[path_index + 1]
            next_position_direction = get_angle(
                return_data['tile_move_to_finished'],
                next_position,
            )
            rotate_relative = next_position_direction - self.subject.direction

        # Check if rotating
        if self.subject.rotate_to != -1:
            # If it is not finished
            if self.subject.start_rotation + self.subject.rotate_duration > now:
                # Let rotation do it's job
                return {
                    'rotate_relative': rotate_relative,
                    'rotate_absolute': next_position_direction,
                    'gui_action': data['gui_action'],
                }
            # rotation finish
            return_data['rotate_to_finished'] = self.subject.rotate_to

        # Check if need to rotate
        if not return_data.get('rotate_to_finished') \
                and self.subject.direction != next_position_direction:
            return_data.update({
                'rotate_relative': rotate_relative,
                'rotate_absolute': next_position_direction,
                'gui_action': data['gui_action'],
            })
            return return_data

        # Need to move to next tile
        return_data['tile_move_to'] = next_position
        return_data['gui_action'] = data['gui_action']
        return return_data

    def action(self, data) -> [Event]:
        events = []
        now = time.time()

        if data.get('path'):
            move = self.subject.intentions.get(MoveToIntention)
            move.path = data['path']
            self.subject.intentions.set(move)

        if data.get('tile_move_to_finished'):
            self.subject.position = data['tile_move_to_finished']
            self.subject.moving_to = (-1, -1)
            self.subject.start_move = -1
            self.subject.move_duration = -1
            events.append(SubjectFinishTileMoveEvent(
                subject_id=self.subject.id,
                move_to=data['tile_move_to_finished'],
                gui_action=data['gui_action'],
            ))

        if data.get('move_to_finished'):
            self.subject.position = data['move_to_finished']
            self.subject.moving_to = (-1, -1)
            self.subject.start_move = -1
            self.subject.move_duration = -1
            self.subject.intentions.remove(MoveToIntention)
            events.append(SubjectFinishMoveEvent(
                subject_id=self.subject.id,
                move_to=data['move_to_finished'],
                gui_action=data['gui_action'],
            ))

        if data.get('rotate_to_finished'):
            self.subject.rotate_to = -1
            self.subject.rotate_duration = -1
            self.subject.start_rotation = -1
            self.subject.direction = data['rotate_to_finished']

            events.append(SubjectFinishRotationEvent(
                subject_id=self.subject.id,
                rotation_absolute=data['rotate_to_finished'],
                gui_action=data['gui_action'],
            ))

        if data.get('rotate_relative'):
            # Test if rotation is already started
            if self.subject.rotate_to == data['rotate_absolute']:
                # look at progression
                rotate_since = now - self.subject.start_rotation
                rotate_progress = rotate_since / self.subject.rotate_duration
                rotation_to_do = self.subject.rotate_to - self.subject.direction
                rotation_done = rotation_to_do * rotate_progress
                self.subject.direction = self.subject.direction + rotation_done
                rotation_left = self.subject.rotate_to - self.subject.direction
                duration = self.subject.get_rotate_duration(angle=rotation_left)
                self.subject.rotate_duration = duration
                self.subject.start_rotation = now

                return [SubjectContinueRotationEvent(
                    subject_id=self.subject.id,
                    rotate_relative=rotation_left,
                    duration=duration,
                    gui_action=data['gui_action'],
                )]
            else:
                duration = self.subject.get_rotate_duration(angle=data['rotate_relative'])
                self.subject.rotate_to = data['rotate_absolute']
                self.subject.rotate_duration = duration
                self.subject.start_rotation = time.time()

                events.append(SubjectStartRotationEvent(
                    subject_id=self.subject.id,
                    rotate_relative=data['rotate_relative'],
                    rotate_absolute=data['rotate_absolute'],
                    duration=duration,
                    gui_action=data['gui_action'],
                ))

        if data.get('tile_move_to'):
            # It is already moving ?
            if self.subject.moving_to == data.get('tile_move_to'):
                # look at progression
                move_since = now - self.subject.start_move
                move_progress = move_since / self.subject.move_duration
                move_done = self.subject.move_duration * move_progress
                duration = self.subject.move_duration - move_done
                self.subject.move_duration = duration

                return [SubjectContinueTileMoveEvent(
                    subject_id=self.subject.id,
                    move_to=data['tile_move_to'],
                    duration=duration,
                    gui_action=data['gui_action'],
                )]
            else:
                move = self.subject.intentions.get(MoveToIntention)
                move_type_duration = self.subject.get_move_duration(move)
                # FIXME: duration depend next tile type, etc
                # see opencombat.gui.base.Game#start_move_subject
                duration = move_type_duration * 1
                self.subject.moving_to = data['tile_move_to']
                self.subject.move_duration = duration
                self.subject.start_move = time.time()
                events.append(SubjectStartTileMoveEvent(
                    subject_id=self.subject.id,
                    move_to=data['tile_move_to'],
                    duration=duration,
                    gui_action=data['gui_action'],
                ))

        return events


class MoveBehaviour(SubjectBehaviour):
    use = [MoveToMechanism]

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.simulation = typing.cast(XYZSimulation, self.simulation)

    def run(self, data) -> object:
        """
        Compute data relative to move
        """
        data = data[MoveToMechanism]
        if not data:
            return False

        # Prepare data
        to = data['to']  # type: typing.Tuple(int, int)
        return_data = {}
        now = time.time()

        # Test if it's first time
        if not data.get('path'):
            return_data['path'] = self.simulation.physics.found_path(
                start=self.subject.position,
                end=to,
                subject=self.subject,
            )
            # find path algorithm can skip start position, add it if not in
            if return_data['path'][0] != self.subject.position:
                return_data['path'] = [self.subject.position] + return_data['path']
            data['path'] = return_data['path']

        # Prepare data
        path = data['path']  # type: typing.List[typing.Tuple(int, int)]
        path_index = path.index(self.subject.position)
        next_position = path[path_index + 1]

        # Test if finish move
        if path_index == len(path) - 1:
            return {
                'move_to_finished': to,
                'gui_action': data['gui_action'],
            }

        # Check if moving
        if self.subject.moving_to == next_position:
            if self.subject.start_move + self.subject.move_duration > now:
                # Let moving
                return {
                    'tile_move_to': next_position,
                    'gui_action': data['gui_action'],
                }
            return_data['tile_move_to_finished'] = self.subject.moving_to
            # Must consider new position of subject
            path_index = path.index(return_data['tile_move_to_finished'])
            if path_index == len(path) - 1:
                return {
                    'move_to_finished': to,
                    'gui_action': data['gui_action'],
                }
            next_position = path[path_index + 1]

        # Need to move to next tile
        return_data['tile_move_to'] = next_position
        return_data['gui_action'] = data['gui_action']
        return return_data

    def action(self, data) -> [Event]:
        events = []
        now = time.time()

        if data.get('path'):
            move = self.subject.intentions.get(MoveToIntention)
            move.path = data['path']
            self.subject.intentions.set(move)

        if data.get('tile_move_to_finished'):
            self.subject.position = data['tile_move_to_finished']
            self.subject.moving_to = (-1, -1)
            self.subject.start_move = -1
            self.subject.move_duration = -1
            events.append(SubjectFinishTileMoveEvent(
                subject_id=self.subject.id,
                move_to=data['tile_move_to_finished'],
                gui_action=data['gui_action'],
            ))

        if data.get('move_to_finished'):
            self.subject.position = data['move_to_finished']
            self.subject.moving_to = (-1, -1)
            self.subject.start_move = -1
            self.subject.move_duration = -1
            self.subject.intentions.remove(MoveToIntention)
            events.append(SubjectFinishMoveEvent(
                subject_id=self.subject.id,
                move_to=data['move_to_finished'],
                gui_action=data['gui_action'],
            ))

        if data.get('tile_move_to'):
            # It is already moving ?
            if self.subject.moving_to == data.get('tile_move_to'):
                # look at progression
                move_since = now - self.subject.start_move
                move_progress = move_since / self.subject.move_duration
                move_done = self.subject.move_duration * move_progress
                duration = self.subject.move_duration - move_done
                self.subject.move_duration = duration
                self.subject.start_move = time.time()

                return [SubjectContinueTileMoveEvent(
                    subject_id=self.subject.id,
                    move_to=data['tile_move_to'],
                    duration=duration,
                    gui_action=data['gui_action'],
                )]
            else:
                move = self.subject.intentions.get(MoveToIntention)
                move_type_duration = self.subject.get_move_duration(move)
                # FIXME: duration depend next tile type, etc
                # see opencombat.gui.base.Game#start_move_subject
                duration = move_type_duration * 1
                self.subject.moving_to = data['tile_move_to']
                self.subject.move_duration = duration
                self.subject.start_move = time.time()
                events.append(SubjectStartTileMoveEvent(
                    subject_id=self.subject.id,
                    move_to=data['tile_move_to'],
                    duration=duration,
                    gui_action=data['gui_action'],
                ))

        return events
