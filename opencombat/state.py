# coding: utf-8
import typing

from _elementtree import Element
from lxml import etree

from synergine2.config import Config
from synergine2.log import get_logger
from synergine2_cocos2d.const import SELECTION_COLOR_RGB

from opencombat.exception import StateLoadError
from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.subject import TileSubject
from opencombat.util import get_class_from_string_path
from opencombat.util import get_text_xml_element
from opencombat.const import FLAG, SIDE
from opencombat.const import FLAG_DE
from opencombat.const import DE_COLOR
from opencombat.const import URSS_COLOR
from opencombat.const import FLAG_URSS


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

            # TODO BS 2018-06-13: Fill subject with property
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

        side = get_text_xml_element(subject_element, 'side')
        if side == 'DE':
            subject_properties.update({
                SELECTION_COLOR_RGB: DE_COLOR,
                FLAG: FLAG_DE,
                SIDE: 'AXIS',
            })
        elif side == 'URSS':
            subject_properties.update({
                SELECTION_COLOR_RGB: URSS_COLOR,
                FLAG: FLAG_URSS,
                SIDE: 'ALLIES',
            })
        else:
            raise NotImplementedError('Don\'t know "{}" side'.format(
                side,
            ))

        subject.properties = subject_properties


class StateLoader(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('StateLoader', config)
        self._config = config
        self._simulation = simulation

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
        # open and read schema file
        schema_file_path = self._config.get(
            'global.state_schema',
            'opencombat/state.xsd',
        )
        with open(schema_file_path, 'r') as schema_file:
            schema_to_check = schema_file.read()

        # open and read xml file
        with open(state_file_path, 'r') as xml_file:
            xml_to_check = xml_file.read()

        xmlschema_doc = etree.fromstring(schema_to_check.encode('utf-8'))
        xmlschema = etree.XMLSchema(xmlschema_doc)

        try:
            doc = etree.fromstring(xml_to_check.encode('utf-8'))
        # check for file IO error
        except IOError as exc:
            self._logger.error(exc)
            raise StateLoadError('Invalid File "{}": {}'.format(
                state_file_path,
                str(exc),
            ))
        # check for XML syntax errors
        except etree.XMLSyntaxError as exc:
            self._logger.error(exc)
            raise StateLoadError('XML Syntax Error in "{}": {}'.format(
                state_file_path,
                str(exc.error_log),
            ))
        except Exception as exc:
            self._logger.error(exc)
            raise StateLoadError('Unknown error with "{}": {}'.format(
                state_file_path,
                str(exc),
            ))

        # validate against schema
        try:
            xmlschema.assertValid(doc)
        except etree.DocumentInvalid as exc:
            self._logger.error(exc)
            raise StateLoadError(
                'Schema validation error with "{}": {}'.format(
                    state_file_path,
                    str(exc),
                )
            )
        except Exception as exc:
            self._logger.error(exc)
            raise StateLoadError(
                'Unknown validation error with "{}": {}'.format(
                    state_file_path,
                    str(exc),
                )
            )

        return doc


class StateLoaderBuilder(object):
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._logger = get_logger('StateLoader', config)
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
