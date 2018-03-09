# coding: utf-8
import pytest
from freezegun import freeze_time
from synergine2_xyz.move.intention import MoveToIntention
from synergine2_xyz.simulation import XYZSimulation

from opencombat.simulation.move import MoveWithRotationBehaviour
from opencombat.simulation.move import MoveToMechanism
from opencombat.simulation.move import MoveBehaviour
from opencombat.simulation.move import SubjectFinishMoveEvent
from opencombat.simulation.move import SubjectStartRotationEvent
from opencombat.simulation.move import SubjectFinishRotationEvent
from opencombat.simulation.move import SubjectContinueRotationEvent
from opencombat.simulation.move import SubjectStartTileMoveEvent
from opencombat.simulation.move import SubjectContinueTileMoveEvent
from opencombat.simulation.move import SubjectFinishTileMoveEvent
from opencombat.simulation.subject import TankSubject
from opencombat.simulation.subject import ManSubject
from opencombat.user_action import UserAction


def test_move_and_rotate_behaviour__begin_rotate(config):
    simulation = XYZSimulation(config)
    simulation.physics.graph.add_edge('0.0', '1.1', {})
    simulation.physics.graph.add_edge('1.1', '2.1', {})

    subject = TankSubject(
        config,
        simulation,
        position=(0, 0),
    )
    move = MoveToIntention(
        to=(2, 1),
        gui_action=UserAction.ORDER_MOVE,
    )
    subject.intentions.set(move)

    move_behaviour = MoveWithRotationBehaviour(
        config=config,
        simulation=simulation,
        subject=subject,
    )

    # Rotation required to begin move
    with freeze_time("2000-01-01 00:00:00", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
            'gui_action': UserAction.ORDER_MOVE,'gui_action': UserAction.ORDER_MOVE,
            'path': [
                (0, 0),
                (1, 1),
                (2, 1),
            ],
            'rotate_relative': 45,
            'rotate_absolute': 45,
        } == data

        events = move_behaviour.action(data)
        assert events
        assert 1 == len(events)
        assert isinstance(events[0], SubjectStartRotationEvent)
        assert 45.0 == events[0].rotate_relative
        assert 4.9995 == events[0].duration
        assert subject.position == (0, 0)
        assert subject.direction == 0
        assert subject.rotate_to == 45
        assert subject.start_rotation == 946684800.0
        assert subject.rotate_duration == 4.9995
        assert subject.intentions.get(MoveToIntention)

    # This is 1 second before end of rotation
    with freeze_time("2000-01-01 00:00:04", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'rotate_relative': 45,
                   'rotate_absolute': 45,
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueRotationEvent)
        assert 9 == round(events[0].rotate_relative)
        assert 0.9995 == events[0].duration
        assert subject.position == (0, 0)
        assert int(subject.direction) == 36
        assert subject.rotate_to == 45
        assert subject.start_rotation == 946684804.0
        assert subject.rotate_duration == 0.9995
        assert subject.intentions.get(MoveToIntention)

    # We are now just after rotation duration, a move will start
    with freeze_time("2000-01-01 00:00:05", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (1, 1),
                   'rotate_to_finished': 45,
        } == data

        events = move_behaviour.action(data)
        assert 2 == len(events)
        assert isinstance(events[1], SubjectStartTileMoveEvent)
        assert isinstance(events[0], SubjectFinishRotationEvent)
        assert (1, 1) == events[1].move_to
        assert 9.0 == events[1].duration
        assert subject.position == (0, 0)
        assert subject.moving_to == (1, 1)
        assert subject.move_duration == 9.0
        assert subject.start_move == 946684805.0
        assert subject.intentions.get(MoveToIntention)

    # We are during the move
    with freeze_time("2000-01-01 00:00:13", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (1, 1),
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueTileMoveEvent)
        assert (1, 1) == events[0].move_to
        assert 1.0 == events[0].duration
        assert subject.intentions.get(MoveToIntention)

    # We are after the move
    with freeze_time("2000-01-01 00:00:14", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to_finished': (1, 1),
                   'rotate_relative': 45,
                   'rotate_absolute': 90,
        } == data

        events = move_behaviour.action(data)
        assert 2 == len(events)
        assert isinstance(events[0], SubjectFinishTileMoveEvent)
        assert isinstance(events[1], SubjectStartRotationEvent)
        assert (1, 1) == events[0].move_to
        assert 4.9995 == events[1].duration
        assert 45 == events[1].rotate_relative
        assert (1, 1) == subject.position
        assert (-1, -1) == subject.moving_to
        assert -1 == subject.start_move
        assert -1 == subject.move_duration
        assert subject.rotate_to == 90
        assert subject.start_rotation == 946684814.0
        assert subject.rotate_duration == 4.9995
        assert subject.intentions.get(MoveToIntention)

    # We are rotating
    with freeze_time("2000-01-01 00:00:18", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'rotate_relative': 45,
                   'rotate_absolute': 90,
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueRotationEvent)
        assert 9 == round(events[0].rotate_relative)
        assert 0.9995 == events[0].duration
        assert subject.position == (1, 1)
        assert int(subject.direction) == 81
        assert subject.rotate_to == 90
        assert subject.start_rotation == 946684818.0
        assert subject.rotate_duration == 0.9995
        assert subject.intentions.get(MoveToIntention)

    # We finish rotating and start to move to final tile
    with freeze_time("2000-01-01 00:00:19", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (2, 1),
                   'rotate_to_finished': 90,
        } == data

        events = move_behaviour.action(data)
        assert 2 == len(events)
        assert isinstance(events[1], SubjectStartTileMoveEvent)
        assert isinstance(events[0], SubjectFinishRotationEvent)
        assert (2, 1) == events[1].move_to
        assert 9.0 == events[1].duration
        assert subject.position == (1, 1)
        assert subject.moving_to == (2, 1)
        assert subject.move_duration == 9.0
        assert subject.start_move == 946684819.0
        assert subject.intentions.get(MoveToIntention)

    # We are moving to final tile
    with freeze_time("2000-01-01 00:00:27", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (2, 1),
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueTileMoveEvent)
        assert (2, 1) == events[0].move_to
        assert 1.0 == events[0].duration
        assert subject.intentions.get(MoveToIntention)

    # We arrived on final tile
    with freeze_time("2000-01-01 00:00:28", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'move_to_finished': (2, 1),
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectFinishMoveEvent)
        assert (2, 1) == events[0].move_to
        assert (2, 1) == subject.position
        assert (-1, -1) == subject.moving_to
        assert -1 == subject.start_move
        assert -1 == subject.move_duration
        with pytest.raises(KeyError):
            assert subject.intentions.get(MoveToIntention)


def test_move_and_rotate_behaviour__begin_move(config):
    simulation = XYZSimulation(config)
    simulation.physics.graph.add_edge('0.0', '0.1', {})
    simulation.physics.graph.add_edge('0.1', '1.1', {})

    subject = TankSubject(
        config,
        simulation,
        position=(0, 0),
    )
    move = MoveToIntention(
        to=(1, 1),
        gui_action=UserAction.ORDER_MOVE,
    )
    subject.intentions.set(move)

    move_behaviour = MoveWithRotationBehaviour(
        config=config,
        simulation=simulation,
        subject=subject,
    )

    # First is a move
    with freeze_time("2000-01-01 00:00:00", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'path': [
                       (0, 0),
                       (0, 1),
                       (1, 1),
                   ],
                   'tile_move_to': (0, 1),
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectStartTileMoveEvent)
        assert (0, 1) == events[0].move_to
        assert 9.0 == events[0].duration
        assert subject.intentions.get(MoveToIntention)

    # Continue the move, rest 1s
    with freeze_time("2000-01-01 00:00:08", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (0, 1),
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueTileMoveEvent)
        assert (0, 1) == events[0].move_to
        assert 1.0 == events[0].duration
        assert subject.intentions.get(MoveToIntention)

    # Tile move finished, begin a rotate
    with freeze_time("2000-01-01 00:00:09", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to_finished': (0, 1),
                   'rotate_relative': 90,
                   'rotate_absolute': 90,
               } == data

        events = move_behaviour.action(data)
        assert 2 == len(events)
        assert isinstance(events[0], SubjectFinishTileMoveEvent)
        assert isinstance(events[1], SubjectStartRotationEvent)
        assert (0, 1) == events[0].move_to
        assert 10 == round(events[1].duration)
        assert 90 == events[1].rotate_relative
        assert (0, 1) == subject.position
        assert (-1, -1) == subject.moving_to
        assert -1 == subject.start_move
        assert -1 == subject.move_duration
        assert subject.rotate_to == 90
        assert subject.start_rotation == 946684809.0
        assert round(subject.rotate_duration) == 10
        assert subject.intentions.get(MoveToIntention)

    # We are rotating, rest 1s
    with freeze_time("2000-01-01 00:00:18", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'rotate_relative': 90,
                   'rotate_absolute': 90,
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueRotationEvent)
        assert 9 == round(events[0].rotate_relative)
        assert 1 == round(events[0].duration)
        assert subject.position == (0, 1)
        assert int(subject.direction) == 81
        assert subject.rotate_to == 90
        assert subject.start_rotation == 946684818.0
        assert round(subject.rotate_duration) == 1
        assert subject.intentions.get(MoveToIntention)

    # We finish rotating and start to move to final tile
    with freeze_time("2000-01-01 00:00:19", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (1, 1),
                   'rotate_to_finished': 90,
               } == data

        events = move_behaviour.action(data)
        assert 2 == len(events)
        assert isinstance(events[1], SubjectStartTileMoveEvent)
        assert isinstance(events[0], SubjectFinishRotationEvent)
        assert (1, 1) == events[1].move_to
        assert 9.0 == events[1].duration
        assert subject.position == (0, 1)
        assert subject.moving_to == (1, 1)
        assert subject.move_duration == 9.0
        assert subject.start_move == 946684819.0
        assert subject.intentions.get(MoveToIntention)

    # Continue the move, rest 1s
    with freeze_time("2000-01-01 00:00:27", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (1, 1),
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueTileMoveEvent)
        assert (1, 1) == events[0].move_to
        assert 1.0 == events[0].duration
        assert subject.moving_to == (1, 1)
        assert subject.move_duration == 1.0
        assert subject.start_move == 946684819.0
        assert subject.intentions.get(MoveToIntention)

    # We arrived on final tile
    with freeze_time("2000-01-01 00:00:28", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'move_to_finished': (1, 1),
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectFinishMoveEvent)
        assert (1, 1) == events[0].move_to
        assert (1, 1) == subject.position
        assert (-1, -1) == subject.moving_to
        assert -1 == subject.start_move
        assert -1 == subject.move_duration
        with pytest.raises(KeyError):
            assert subject.intentions.get(MoveToIntention)


def test_move_behaviour(config):
    simulation = XYZSimulation(config)
    simulation.physics.graph.add_edge('0.0', '0.1', {})
    simulation.physics.graph.add_edge('0.1', '1.1', {})

    subject = ManSubject(
        config,
        simulation,
        position=(0, 0),
    )
    move = MoveToIntention(
        to=(1, 1),
        gui_action=UserAction.ORDER_MOVE,
    )
    subject.intentions.set(move)

    move_behaviour = MoveBehaviour(
        config=config,
        simulation=simulation,
        subject=subject,
    )

    # First begin move
    with freeze_time("2000-01-01 00:00:00", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'path': [
                       (0, 0),
                       (0, 1),
                       (1, 1),
                   ],
                   'tile_move_to': (0, 1),
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectStartTileMoveEvent)
        assert (0, 1) == events[0].move_to
        assert 3.0 == events[0].duration
        assert subject.intentions.get(MoveToIntention)

    # Continue the move, rest 1s
    with freeze_time("2000-01-01 00:00:02", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (0, 1),
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueTileMoveEvent)
        assert (0, 1) == events[0].move_to
        assert 1.0 == events[0].duration
        assert subject.intentions.get(MoveToIntention)

    # Tile move finished, begin a new move
    with freeze_time("2000-01-01 00:00:03", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to_finished': (0, 1),
                   'tile_move_to': (1, 1),
               } == data

        events = move_behaviour.action(data)
        assert 2 == len(events)
        assert isinstance(events[0], SubjectFinishTileMoveEvent)
        assert isinstance(events[1], SubjectStartTileMoveEvent)
        assert (0, 1) == events[0].move_to
        assert (1, 1) == events[1].move_to
        assert 3 == events[1].duration
        assert (0, 1) == subject.position
        assert (1, 1) == subject.moving_to
        assert 946684803.0 == subject.start_move
        assert 3 == subject.move_duration
        assert subject.intentions.get(MoveToIntention)

    # We are moving, rest 1s
    with freeze_time("2000-01-01 00:00:05", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'tile_move_to': (1, 1),
               } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectContinueTileMoveEvent)
        assert (1, 1) == events[0].move_to
        assert 1.0 == events[0].duration
        assert (0, 1) == subject.position
        assert (1, 1) == subject.moving_to
        assert 946684805.0 == subject.start_move
        assert 1 == subject.move_duration
        assert subject.intentions.get(MoveToIntention)

    # We arrived on final tile
    with freeze_time("2000-01-01 00:00:06", tz_offset=0):
        data = move_behaviour.run({MoveToMechanism: move.get_data()})
        assert {
                   'gui_action': UserAction.ORDER_MOVE,
                   'move_to_finished': (1, 1),
        } == data

        events = move_behaviour.action(data)
        assert 1 == len(events)
        assert isinstance(events[0], SubjectFinishMoveEvent)
        assert (1, 1) == events[0].move_to
        assert (1, 1) == subject.position
        assert (-1, -1) == subject.moving_to
        assert -1 == subject.start_move
        assert -1 == subject.move_duration
        with pytest.raises(KeyError):
            assert subject.intentions.get(MoveToIntention)
