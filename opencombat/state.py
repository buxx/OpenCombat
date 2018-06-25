# coding: utf-8
import typing

from _elementtree import Element
from lxml import etree

from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.exception import NotFoundError
from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.subject import TileSubject
from opencombat.util import get_class_from_string_path
from opencombat.util import pretty_xml
from opencombat.util import get_text_xml_element
from opencombat.xml import XmlValidator
from opencombat.const import SIDE
from opencombat.const import FLAG
from opencombat.const import SELECTION_COLOR_RGB


class State(object):
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

    @property
    def subjects(self) -> typing.List[TileSubject]:
        if self._subjects is None:
            self._subjects = self._get_subjects()

        return self._subjects

    def _get_subjects(self) -> typing.List[TileSubject]:
        subjects = []
        subject_elements = self._state_root.find('subjects').findall('subject')

        for subject_element in subject_elements:
            subject_class_path = subject_element.find('type').text
            subject_class = get_class_from_string_path(
                self._config,
                subject_class_path,
            )
            subject = subject_class(self._config, self._simulation)
            self._fill_subject(subject, subject_element)
            subjects.append(subject)

        return subjects

    def _fill_subject(
        self,
        subject: TileSubject,
        subject_element: Element,
    ) -> None:
        subject_properties = {}

        subject.position = tuple(
            map(
                int,
                get_text_xml_element(subject_element, 'position').split(','),
            ),
        )
        subject.direction = float(
            get_text_xml_element(subject_element, 'direction'),
        )
        # TODO BS 2018-06-20: Maybe need apply this mode no ?
        subject.combat_mode = \
            get_text_xml_element(subject_element, 'combat_mode')

        properties_element = subject_element.find('properties')
        decode_properties_map = self._get_decode_properties_map()

        for item_element in properties_element.findall('item'):
            key_text = item_element.find('key').text
            value_text = item_element.find('value').text

            try:
                decoded_value = decode_properties_map[key_text](value_text)
            except KeyError:
                raise NotFoundError(
                    'You try to load property "{}" but it is unknown'.format(
                        key_text,
                    )
                )

            subject_properties[key_text] = decoded_value

        subject.properties = subject_properties

    def _get_decode_properties_map(self) -> typing.Dict[str, typing.Callable[[str], typing.Any]]:  # nopep8
        return {
            SELECTION_COLOR_RGB: lambda v: tuple(map(int, v.split(','))),
            FLAG: str,
            SIDE: str,
        }


class StateDumper(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('StateDumper', config)
        self._config = config
        self._simulation = simulation

        state_template = self._config.resolve(
           'global.state_template',
           'opencombat/state_template.xml',
        )
        with open(state_template, 'r') as xml_file:
            template_str = xml_file.read()

        parser = etree.XMLParser(remove_blank_text=True)
        self._state_root = etree.fromstring(
            template_str.encode('utf-8'),
            parser,
        )
        self._state_root_filled = False

    def get_state_dump(self) -> str:
        if not self._state_root_filled:
            self._fill_state_root()

        return pretty_xml(
            etree.tostring(
                self._state_root,
            ).decode('utf-8'),
        )

    def _fill_state_root(self) -> None:
        subjects_element = self._state_root.find('subjects')
        map_element = self._state_root.find('map')

        map_name_element = etree.SubElement(map_element, 'name')
        map_name_element.text = self._config.resolve('_runtime.map_dir_path')

        for subject in self._simulation.subjects:
            subject_element = etree.SubElement(subjects_element, 'subject')

            position_element = etree.SubElement(subject_element, 'type')
            position_element.text = '.'.join([
                subject.__module__,
                subject.__class__.__name__,
            ])

            position_element = etree.SubElement(subject_element, 'position')
            position_element.text = ','.join(map(str, subject.position))

            direction_element = etree.SubElement(subject_element, 'direction')
            direction_element.text = str(subject.direction)

            combat_mode_element = etree.SubElement(
                subject_element,
                'combat_mode',
            )
            combat_mode_element.text = str(subject.combat_mode)

            properties_element = etree.SubElement(
                subject_element,
                'properties',
            )
            encode_properties_map = self._get_encode_properties_map()

            for key, value in subject.properties.items():
                item_element = etree.SubElement(properties_element, 'item')
                key_element = etree.SubElement(item_element, 'key')
                value_element = etree.SubElement(item_element, 'value')

                key_element.text = str(key)
                value_element.text = encode_properties_map[key](value)

        self._state_root_filled = True

    def _get_encode_properties_map(self) -> typing.Dict[str, typing.Callable[[typing.Any], str]]:  # nopep8:
        return {
            SELECTION_COLOR_RGB: lambda v: ','.join(map(str, v)),
            FLAG: str,
            SIDE: str,
        }


class StateLoader(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('StateLoader', config)
        self._config = config
        self._simulation = simulation

        schema_file_path = self._config.get(
            'global.state_schema',
            'opencombat/state.xsd',
        )
        self._xml_validator = XmlValidator(
            config,
            schema_file_path,
        )

    def get_state(
        self,
        state_file_path: str,
    ) -> State:
        return State(
            self._config,
            self._validate_and_return_state_element(state_file_path),
            self._simulation,
        )

    def _validate_and_return_state_element(
        self,
        state_file_path: str,
    ) -> Element:
        return self._xml_validator.validate_and_return(state_file_path)


class StateConstructorBuilder(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('StateConstructorBuilder', config)
        self._config = config
        self._simulation = simulation

    def get_state_loader(
        self,
    ) -> StateLoader:
        class_address = self._config.resolve(
            'global.state_loader',
            'opencombat.state.StateLoader',
        )
        state_loader_class = get_class_from_string_path(
            self._config,
            class_address,
        )
        return state_loader_class(
            self._config,
            self._simulation,
        )

    def get_state_dumper(
        self,
    ) -> StateDumper:
        class_address = self._config.resolve(
            'global.state_dumper',
            'opencombat.state.StateDumper',
        )
        state_loader_class = get_class_from_string_path(
            self._config,
            class_address,
        )
        return state_loader_class(
            self._config,
            self._simulation,
        )
