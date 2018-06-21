# coding: utf-8
import typing

from synergine2.config import Config

from opencombat.strategy.team.model import UnitModel
from opencombat.xml import XmlValidator


class UnitStash(object):
    def __init__(
        self,
        config: Config,
        units_file_path: str,
    ) -> None:
        self._config = config
        self._units = None  # typing.List[UnitModel]

        schema_file_path = self._config.get(
            'global.teams_schema',
            'opencombat/strategy/teams.xsd',
        )
        self._xml_validator = XmlValidator(
            config,
            schema_file_path,
        )

    def get_units(self) -> typing.List[UnitModel]:
        pass

    def get_unit(self, unit_id: str) -> UnitModel:
        pass
