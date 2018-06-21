# coding: utf-8
import typing

from synergine2.config import Config

from opencombat.strategy.unit.model import TeamModel
from opencombat.xml import XmlValidator


class TeamStash(object):
    def __init__(
        self,
        config: Config,
        teams_file_path: str,
    ) -> None:
        self._config = config
        self._teams = None  # typing.List[TeamModel]

        schema_file_path = self._config.get(
            'global.teams_schema',
            'opencombat/strategy/teams.xsd',
        )
        self._xml_validator = XmlValidator(
            config,
            schema_file_path,
        )

    def get_teams(self) -> typing.List[TeamModel]:
        pass

    def get_team(self, unit_id: str) -> TeamModel:
        pass
