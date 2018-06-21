# coding: utf-8
import typing

from opencombat.simulation.subject import TileSubject


class UnitModel(object):
    def __init__(
        self,
        id_: str,
        name: str,
        class_: typing.Type[TileSubject],
        country: str,
    ) -> None:
        self._id = id_
        self._name = name
        self._class = class_
        self._country = country

    @property
    def id(self) -> str:
        return self._id

    @property
    def name(self) -> str:
        return self._name

    @property
    def class_(self) -> typing.Type[TileSubject]:
        return self._class

    @property
    def country(self) -> str:
        return self._country
