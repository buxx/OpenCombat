# coding: utf-8
import typing
from _elementtree import Element

from synergine2.config import Config

from opencombat.exception import NotFoundException
from opencombat.strategy.team.model import TeamModel
from opencombat.strategy.unit.stash import UnitStash
from opencombat.util import get_text_xml_element
from opencombat.xml import XmlValidator


class TeamStash(object):
    def __init__(
        self,
        config: Config,
        teams_file_path: str,
        unit_stash: UnitStash,
    ) -> None:
        self._config = config
        self._teams = None  # type: typing.List[TeamModel]
        self._unit_stash = unit_stash

        self.schema_file_path = self._config.get(
            'global.teams_schema',
            'opencombat/strategy/teams.xsd',
        )
        self._xml_validator = XmlValidator(
            config,
            self.schema_file_path,
        )
        self._root_element = self._xml_validator.validate_and_return(
            teams_file_path,
        )

    def _get_computed_teams(self) -> typing.List[TeamModel]:
        teams = []

        for team_element in self._root_element.findall('team'):
            team_element = typing.cast(Element, team_element)

            team_id = team_element.attrib['id']
            team_country = team_element.attrib['country']
            team_name = get_text_xml_element(team_element, 'name')
            team_units = []

            units_element = team_element.find('units')
            for unit_element in units_element.findall('unit'):
                unit_id = get_text_xml_element(unit_element, 'id')
                unit = self._unit_stash.get_unit(unit_id, team_country)
                team_units.append(unit)

            teams.append(
                TeamModel(
                    id_=team_id,
                    country=team_country,
                    name=team_name,
                    units=team_units
                )
            )

        return teams

    @property
    def teams(self) -> typing.List[TeamModel]:
        if self._teams is None:
            self._teams = self._get_computed_teams()

        return self._teams

    def get_team(
        self,
        team_id: str,
        team_country: str,
    ) -> TeamModel:
        for team in self.teams:
            if team.id == team_id and team.country == team_country:
                return team

        raise NotFoundException(
            'No team matching with id "{}" and country "{}" in "{}"'.format(
                team_id,
                team_country,
                self.schema_file_path,
            )
        )
