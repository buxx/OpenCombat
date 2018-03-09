# coding: utf-8
import typing

from synergine2.simulation import SubjectBehaviourSelector
from synergine2.simulation import SubjectBehaviour
from opencombat.user_action import UserAction
from synergine2_xyz.move.intention import MoveToIntention

from opencombat.const import COLLECTION_ALIVE
from opencombat.const import COMBAT_MODE_DEFENSE
from opencombat.simulation.base import BaseSubject
from opencombat.simulation.move import MoveBehaviour
from opencombat.simulation.move import MoveWithRotationBehaviour
from opencombat.simulation.behaviour import EngageOpponent
from opencombat.simulation.behaviour import LookAroundBehaviour
from synergine2.share import shared


class TileBehaviourSelector(SubjectBehaviourSelector):
    def reduce_behaviours(
        self,
        behaviours: typing.Dict[typing.Type[SubjectBehaviour], object],
    ) -> typing.Dict[typing.Type[SubjectBehaviour], object]:
        return behaviours


class TileSubject(BaseSubject):
    start_collections = [
        COLLECTION_ALIVE,
    ]
    visible_opponent_ids = shared.create_self('visible_opponent_ids', lambda: [])
    combat_mode = shared.create_self('combat_mode', COMBAT_MODE_DEFENSE)
    behaviour_selector_class = TileBehaviourSelector

    direction = shared.create_self('direction', 0)
    moving_to = shared.create_self('moving_to', (-1, -1))
    move_duration = shared.create_self('move_duration', -1)
    start_move = shared.create_self('start_move', -1)

    rotate_to = shared.create_self('rotate_to', -1)
    rotate_duration = shared.create_self('rotate_duration', -1)
    start_rotation = shared.create_self('start_rotation', -1)

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._walk_ref_time = float(self.config.resolve('game.move.walk_ref_time'))
        self._run_ref_time = float(self.config.resolve('game.move.run_ref_time'))
        self._crawl_ref_time = float(self.config.resolve('game.move.crawl_ref_time'))
        self._rotate_ref_time = float(self.config.resolve('game.move.rotate_ref_time'))
        self.direction = kwargs.get('direction', 0)

    @property
    def global_move_coeff(self) -> float:
        return 1

    @property
    def run_duration(self) -> float:
        """
        :return: move to tile time (s) when running
        """
        return self._run_ref_time * self.global_move_coeff

    @property
    def walk_duration(self) -> float:
        """
        :return: move to tile time (s) when walking
        """
        return self._walk_ref_time * self.global_move_coeff

    @property
    def crawl_duration(self) -> float:
        """
        :return: move to tile time (s) when crawling
        """
        return self._crawl_ref_time * self.global_move_coeff

    def get_rotate_duration(self, angle: float) -> float:
        return angle * self._rotate_ref_time

    def get_move_duration(self, move: MoveToIntention) -> float:
        gui_action = move.gui_action

        if gui_action == UserAction.ORDER_MOVE:
            return self.walk_duration
        if gui_action == UserAction.ORDER_MOVE_FAST:
            return self.run_duration
        if gui_action == UserAction.ORDER_MOVE_CRAWL:
            return self.crawl_duration

        raise NotImplementedError(
            'Gui action {} unknown'.format(move.gui_action)
        )


class ManSubject(TileSubject):
    behaviours_classes = [
        MoveBehaviour,
        LookAroundBehaviour,
        EngageOpponent,
    ]  # type: typing.List[SubjectBehaviour]


class TankSubject(TileSubject):
    behaviours_classes = [
        MoveWithRotationBehaviour,
        LookAroundBehaviour,
        EngageOpponent,
    ]  # type: typing.List[SubjectBehaviour]

    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        # TODO BS 2018-01-26: This coeff will be dependent of real
        # unit type (tiger 2, etc)
        self._global_move_coeff = self.config.resolve(
            'game.move.subject.tank1.global_move_coeff',
            3,
        )
        self._rotate_ref_time = float(self.config.resolve(
            'game.move.subject.tank1.rotate_ref_time',
            0.1111,
        ))

    @property
    def global_move_coeff(self) -> float:
        return self._global_move_coeff
