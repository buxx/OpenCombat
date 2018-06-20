# coding: utf-8
import typing

from synergine2.config import Config

from opencombat.strategy.unit.model import TeamModel


class TeamStash(object):
    def __init__(
        self,
        config: Config,
        teams_file_path: str,
    ) -> None:
        self._confg = config
        # TODO Load xml, validate
        self._teams = None  # typing.List[TeamModel]

    def get_teams(self) -> typing.List[TeamModel]:
        pass

    def get_team(self, unit_id: str) -> TeamModel:
        pass
