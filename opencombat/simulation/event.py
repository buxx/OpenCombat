# coding: utf-8
import typing

from synergine2.simulation import Event

from opencombat.const import COLLECTION_ALIVE

if typing.TYPE_CHECKING:
    from opencombat.simulation.subject import TileSubject

DEFAULT_WEAPON_TYPE = 'DEFAULT_WEAPON_TYPE'


class NewVisibleOpponent(Event):
    def __init__(
        self,
        observer_subject_id: int,
        observed_subject_id: int,
    ) -> None:
        self.observer_subject_id = observer_subject_id
        self.observed_subject_id = observed_subject_id


class NoLongerVisibleOpponent(Event):
    def __init__(
        self,
        observer_subject_id: int,
        observed_subject_id: int,
    ) -> None:
        self.observer_subject_id = observer_subject_id
        self.observed_subject_id = observed_subject_id


class FireEvent(Event):
    def __init__(
        self,
        shooter_subject_id: int,
        target_position: typing.Tuple[int, int],
        weapon_type: str=DEFAULT_WEAPON_TYPE,
    ) -> None:
        self.shooter_subject_id = shooter_subject_id
        self.target_position = target_position
        self.weapon_type = weapon_type


class DieEvent(Event):
    @classmethod
    def apply_subject_death(cls, subject: 'TileSubject') -> None:
        subject.remove_collection(COLLECTION_ALIVE)
        subject.intentions.remove_all()

    def __init__(
        self,
        shooter_subject_id: int,
        shoot_subject_id: int,
    ) -> None:
        self.shooter_subject_id = shooter_subject_id
        self.shoot_subject_id = shoot_subject_id
