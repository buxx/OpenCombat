# coding: utf-8


# TODO: Reprendre les events Move, pour les lister tous ici
import typing

from synergine2.simulation import Event


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
    ) -> None:
        self.shooter_subject_id = shooter_subject_id
        self.target_position = target_position


class DieEvent(Event):
    def __init__(
        self,
        shooter_subject_id: int,
        shoot_subject_id: int,
    ) -> None:
        self.shooter_subject_id = shooter_subject_id
        self.shoot_subject_id = shoot_subject_id
