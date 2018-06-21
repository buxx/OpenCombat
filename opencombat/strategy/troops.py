# coding: utf-8
from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.strategy.unit.stash import UnitStash
from opencombat.util import get_class_from_string_path


class TroopClassBuilder(object):
    def __init__(
        self,
        config: Config,
    ) -> None:
        self._logger = get_logger('TroopManagerBuilder', config)
        self._config = config

    def get_unit_stash(
        self,
        units_file_path: str,
    ) -> UnitStash:
        class_address = self._config.resolve(
            'global.unit_stash',
            'opencombat.strategy.unit.stash.UnitStash',
        )
        class_ = get_class_from_string_path(
            self._config,
            class_address,
        )
        return class_(
            self._config,
            units_file_path,
        )

    def get_team_stash(
        self,
        units_file_path: str,
        teams_file_path: str,
    ) -> UnitStash:
        class_address = self._config.resolve(
            'global.team_stash',
            'opencombat.strategy.team.stash.TeamStash',
        )
        class_ = get_class_from_string_path(
            self._config,
            class_address,
        )

        unit_stash = self.get_unit_stash(units_file_path)
        return class_(
            self._config,
            teams_file_path,
            unit_stash=unit_stash,
        )
