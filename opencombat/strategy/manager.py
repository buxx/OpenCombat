# coding: utf-8
from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.strategy.troops import TroopClassBuilder


class TroopManager(object):
    def __init__(
        self,
        config: Config,
        units_file_path: str,
        teams_file_path: str,
    ) -> None:
        self._config = config
        self._logger = get_logger('TroopManager', config)

        builder = TroopClassBuilder(config)
        self._unit_stash = builder.get_unit_stash(units_file_path)
        self._team_stash = builder.get_team_stash(
            units_file_path,
            teams_file_path,
        )
