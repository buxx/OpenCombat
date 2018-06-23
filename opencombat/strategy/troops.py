# coding: utf-8
import typing

from lxml import etree

from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.strategy.team.model import TeamModel
from opencombat.strategy.team.stash import TeamStash
from opencombat.strategy.unit.stash import UnitStash
from opencombat.util import get_class_from_string_path, pretty_xml


class TroopDumper(object):
    def __init__(
        self,
        config: Config,
    ) -> None:
        self._config = config
        self._logger = get_logger('TroopDumper', config)

    def get_troop_dump(
        self,
        countries_troops: typing.Dict[str, typing.List[TeamModel]],
    ) -> str:
        troops_template = self._config.resolve(
            'global.troops_template',
            'opencombat/strategy/troops_template.xml',
        )
        with open(troops_template, 'r') as xml_file:
            template_str = xml_file.read()

        parser = etree.XMLParser(remove_blank_text=True)
        state_root = etree.fromstring(
            template_str.encode('utf-8'),
            parser,
        )

        for country, teams in countries_troops.items():
            for team in teams:
                troop_element = etree.SubElement(state_root, 'troop')
                troop_element.attrib['country'] = country
                troop_element.attrib['team_id'] = team.id

        return pretty_xml(
            etree.tostring(
                state_root,
            ).decode('utf-8'),
        )


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
    ) -> TeamStash:
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

    def get_troop_dumper(self) -> TroopDumper:
        class_address = self._config.resolve(
            'global.troop_dumper',
            'opencombat.strategy.troops.TroopDumper',
        )
        class_ = get_class_from_string_path(
            self._config,
            class_address,
        )

        return class_(
            self._config,
        )
