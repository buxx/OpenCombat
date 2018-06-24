# coding: utf-8
import typing
from _elementtree import Element

from lxml import etree

from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.subject import TileSubject
from opencombat.strategy.team.model import TeamModel
from opencombat.strategy.team.stash import TeamStash
from opencombat.strategy.unit.stash import UnitStash
from opencombat.util import get_class_from_string_path, pretty_xml
from opencombat.xml import XmlValidator


class Troop(object):
    def __init__(
        self,
        config: Config,
        state_root: Element,
        simulation: TileStrategySimulation,
    ) -> None:
        self._config = config
        self._state_root = state_root
        self._subjects = None  # type: typing.List[TileSubject]
        self._simulation = simulation
        self._builder = TroopClassBuilder(config)

    @property
    def subjects(self) -> typing.List[TileSubject]:
        if self._subjects is None:
            self._subjects = self.get_computed_subjects()

        return self._subjects

    def get_computed_subjects(self) -> typing.List[TileSubject]:
        units_file_path = self._config.get(
            'global.units',
            'opencombat/strategy/units.xml',
        )
        teams_file_path = self._config.get(
            'global.teams',
            'opencombat/strategy/teams.xml',
        )

        team_stash = self._builder.get_team_stash(
            units_file_path,
            teams_file_path,
        )

        # Parse team, build Subjects
        subjects = []  # type: typing.List[TileSubject]
        for troop in self._state_root.findall('troop'):
            country = troop.attrib['country']
            team_id = troop.attrib['team_id']
            team = team_stash.get_team(team_id, country)

            for unit in team.units:
                subject = unit.class_(self._config, self._simulation)
                subjects.append(subject)

        # TODO BS 2018-06-25: place subjects on map, set side, color, etc
        return subjects


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


class TroopLoader(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('TroopLoader', config)
        self._config = config
        self._simulation = simulation

        schema_file_path = self._config.get(
            'global.troop_schema',
            'opencombat/strategy/troops.xsd',
        )
        self._xml_validator = XmlValidator(
            config,
            schema_file_path,
        )

    def get_troop(
        self,
        troop_file_path: str,
    ) -> Troop:
        return Troop(
            self._config,
            self._validate_and_return_state_element(troop_file_path),
            self._simulation,
        )

    def _validate_and_return_state_element(
        self,
        troop_file_path: str,
    ) -> Element:
        return self._xml_validator.validate_and_return(troop_file_path)


class TroopConstructorBuilder(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('TroopConstructorBuilder', config)
        self._config = config
        self._simulation = simulation

    def get_troop_loader(
        self,
    ) -> TroopLoader:
        class_address = self._config.resolve(
            'global.troop_loader',
            'opencombat.strategy.troops.TroopLoader',
        )
        troop_loader_class = get_class_from_string_path(
            self._config,
            class_address,
        )
        return troop_loader_class(
            self._config,
            self._simulation,
        )
