# coding: utf-8
import typing

from synergine2.simulation import SubjectBehaviourSelector, SubjectBehaviour

from opencombat.const import COLLECTION_ALIVE
from opencombat.const import COMBAT_MODE_DEFENSE
from opencombat.simulation.base import BaseSubject
from opencombat.simulation.behaviour import MoveToBehaviour
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
    behaviours_classes = [
        MoveToBehaviour,
        LookAroundBehaviour,
        EngageOpponent,
    ]
    visible_opponent_ids = shared.create_self('visible_opponent_ids', lambda: [])
    combat_mode = shared.create_self('combat_mode', COMBAT_MODE_DEFENSE)
    behaviour_selector_class = TileBehaviourSelector

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._walk_ref_time = float(self.config.resolve('game.move.walk_ref_time'))
        self._run_ref_time = float(self.config.resolve('game.move.run_ref_time'))
        self._crawl_ref_time = float(self.config.resolve('game.move.crawl_ref_time'))

    @property
    def global_move_coeff(self) -> float:
        return 1

    @property
    def run_duration(self) -> float:
        return self._run_ref_time * self.global_move_coeff

    @property
    def walk_duration(self) -> float:
        return self._walk_ref_time * self.global_move_coeff

    @property
    def crawl_duration(self) -> float:
        return self._crawl_ref_time * self.global_move_coeff


class ManSubject(TileSubject):
    pass

class TankSubject(TileSubject):
    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        # TODO BS 2018-01-26: This coeff will be dependent of real
        # unit type (tiger 2, etc)
        self._global_move_coeff = self.config.resolve(
            'game.move.subject.tank1.global_move_coeff',
            3,
        )

    @property
    def global_move_coeff(self) -> float:
        return self._global_move_coeff
