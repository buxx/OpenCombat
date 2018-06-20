# coding: utf-8
import typing

from synergine2.config import Config

from opencombat.strategy.team.model import UnitModel


class UnitStash(object):
    def __init__(
        self,
        config: Config,
        units_file_path: str,
    ) -> None:
        self._confg = config
        # TODO Load xml, validate
        self._units = None  # typing.List[UnitModel]

    def get_units(self) -> typing.List[UnitModel]:
        pass

    def get_unit(self, unit_id: str) -> UnitModel:
        pass
