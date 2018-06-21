# coding: utf-8
import typing
from _elementtree import Element

from synergine2.config import Config

from opencombat.exception import NotFoundException
from opencombat.strategy.team.model import UnitModel
from opencombat.util import get_text_xml_element
from opencombat.util import get_class_from_string_path
from opencombat.xml import XmlValidator


class UnitStash(object):
    def __init__(
        self,
        config: Config,
        units_file_path: str,
    ) -> None:
        self._config = config
        self._units = None  # type: typing.List[UnitModel]

        self.schema_file_path = self._config.get(
            'global.teams_schema',
            'opencombat/strategy/units.xsd',
        )
        self._xml_validator = XmlValidator(
            config,
            self.schema_file_path,
        )
        self._root_element = self._xml_validator.validate_and_return(
            units_file_path,
        )

    def _get_computed_units(self) -> typing.List[UnitModel]:
        units = []

        for unit_element in self._root_element.findall('unit'):
            unit_element = typing.cast(Element, unit_element)

            unit_id = unit_element.attrib['id']
            unit_country = unit_element.attrib['country']
            unit_name = get_text_xml_element(unit_element, 'name')
            unit_class_path = get_text_xml_element(unit_element, 'type')
            unit_class = get_class_from_string_path(
                self._config,
                unit_class_path,
            )

            units.append(
                UnitModel(
                    id_=unit_id,
                    name=unit_name,
                    class_=unit_class,
                    country=unit_country,
                )
            )

        return units

    @property
    def units(self) -> typing.List[UnitModel]:
        if self._units is None:
            self._units = self._get_computed_units()

        return self._units

    def get_unit(
        self,
        unit_id: str,
        unit_country: str,
    ) -> UnitModel:
        for unit in self.units:
            if unit.id == unit_id and unit.country == unit_country:
                return unit

        raise NotFoundException(
            'No unit matching with id "{}" and country "{}" in "{}"'.format(
                unit_id,
                unit_country,
                self.schema_file_path,
            )
        )
