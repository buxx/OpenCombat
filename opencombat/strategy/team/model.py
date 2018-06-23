# coding: utf-8
import typing

from opencombat.strategy.unit.model import UnitModel


class TeamModel(object):
    def __init__(
        self,
        id_: str,
        name: str,
        country: str,
        units: typing.List[UnitModel],
    ) -> None:
        self._id = id_
        self._name = name
        self._country = country
        self._units = units

    @property
    def id(self) -> str:
        return self._id

    @property
    def name(self) -> str:
        return self._name

    @property
    def country(self) -> str:
        return self._country

    @property
    def units(self) -> typing.List[UnitModel]:
        return self._units
