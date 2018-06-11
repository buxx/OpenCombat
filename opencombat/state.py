# coding: utf-8
import importlib
import typing
from io import StringIO
import sys

from lxml import etree

from synergine2.config import Config
from synergine2.log import get_logger
from synergine2.simulation import Subject

from opencombat.exception import StateLoadError


class StateLoader(object):
    def __init__(
        self,
        config: Config,
        state_file_path: str,
    ) -> None:
        self.logger = get_logger('StateLoader', config)
        self.config = config
        self.state_file_path = state_file_path
        self._validate()

    def _validate(self) -> None:
        # open and read schema file
        schema_file_path = self.config.get(
            'global.state_schema',
            'opencombat/state.xsd',
        )
        with open(schema_file_path, 'r') as schema_file:
            schema_to_check = schema_file.read()

        # open and read xml file
        with open(self.state_file_path, 'r') as xml_file:
            xml_to_check = xml_file.read()

        xmlschema_doc = etree.parse(StringIO(schema_to_check))
        xmlschema = etree.XMLSchema(xmlschema_doc)

        try:
            doc = etree.parse(StringIO(xml_to_check))
        # check for file IO error
        except IOError as exc:
            self.logger.error(exc)
            raise StateLoadError('Invalid File "{}": {}'.format(
                self.state_file_path,
                str(exc),
            ))
        # check for XML syntax errors
        except etree.XMLSyntaxError as exc:
            self.logger.error(exc)
            raise StateLoadError('XML Syntax Error in "{}": {}'.format(
                self.state_file_path,
                str(exc.error_log),
            ))
        except Exception as exc:
            self.logger.error(exc)
            raise StateLoadError('Unknown error with "{}": {}'.format(
                self.state_file_path,
                str(exc),
            ))

        # validate against schema
        try:
            xmlschema.assertValid(doc)
        except etree.DocumentInvalid as exc:
            self.logger.error(exc)
            raise StateLoadError(
                'Schema validation error with "{}": {}'.format(
                    self.state_file_path,
                    str(exc),
                )
            )
        except Exception as exc:
            self.logger.error(exc)
            raise StateLoadError(
                'Unknown validation error with "{}": {}'.format(
                    self.state_file_path,
                    str(exc),
                )
            )

    def get_subjects(self) -> typing.List[Subject]:
        raise NotImplementedError('TODO')


class StateLoaderBuilder(object):
    def __init__(
        self,
        config: Config,
    ) -> None:
        self.logger = get_logger('StateLoader', config)
        self.config = config

    def get_state_loader(
        self,
        state_file_path: str,
    ) -> StateLoader:
        class_address = self.config.get(
            'global.state_loader',
            'opencombat.state.StateLoader',
        )
        module_address = '.'.join(class_address.split('.')[:-1])
        class_name = class_address.split('.')[-1]
        module_ = importlib.import_module(module_address)
        state_loader_class = getattr(module_, class_name)
        return state_loader_class(self.config, state_file_path)
