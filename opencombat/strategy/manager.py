# coding: utf-8
import typing

from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.strategy.team.model import TeamModel
from opencombat.strategy.team.stash import TeamStash
from opencombat.strategy.troops import TroopClassBuilder
from opencombat.strategy.unit.stash import UnitStash


class TroopManager(object):
    def __init__(
        self,
        config: Config,
        units_file_path: str,
        teams_file_path: str,
    ) -> None:
        self._config = config
        self._logger = get_logger('TroopManager', config)

        self._builder = TroopClassBuilder(config)
        self._unit_stash = self._builder.get_unit_stash(
            units_file_path,
        )
        self._team_stash = self._builder.get_team_stash(
            units_file_path,
            teams_file_path,
        )

    @property
    def team_stash(self) -> TeamStash:
        return self._team_stash

    @property
    def unit_stash(self) -> UnitStash:
        return self._unit_stash

    def get_troop_dump(
        self,
        countries_troops: typing.Dict[str, typing.List[TeamModel]],
    ):
        dumper = self._builder.get_troop_dumper()
        return dumper.get_troop_dump(countries_troops)
