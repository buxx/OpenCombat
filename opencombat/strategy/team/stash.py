# coding: utf-8
import typing

from opencombat.strategy.unit.model import TeamModel


class TeamStash(object):
    def __init__(
        self,
        teams_file_path: str,
    ) -> None:
        # TODO Load xml, validate
        self._teams = None  # typing.List[TeamModel]

    def get_teams(self) -> typing.List[TeamModel]:
        pass

    def get_team(self, unit_id: str) -> TeamModel:
        pass
