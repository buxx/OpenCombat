# coding: utf-8
from _elementtree import Element

from lxml import etree
from synergine2.config import Config
from synergine2.log import get_logger

from opencombat.exception import StateLoadError


class XmlValidator(object):
    def __init__(
        self,
        config: Config,
        schema_file_path: str,
    ) -> None:
        self._config = config
        self._logger = get_logger('XmlValidator', config)
        self._schema_file_path = schema_file_path

    def validate_and_return(self, xml_file_path: str) -> Element:
        with open(self._schema_file_path, 'r') as schema_file:
            schema_to_check = schema_file.read()

        # open and read xml file
        with open(xml_file_path, 'r') as xml_file:
            xml_to_check = xml_file.read()

        xmlschema_doc = etree.fromstring(schema_to_check.encode('utf-8'))
        xmlschema = etree.XMLSchema(xmlschema_doc)

        try:
            doc = etree.fromstring(xml_to_check.encode('utf-8'))
        # check for file IO error
        except IOError as exc:
            self._logger.error(exc)
            raise StateLoadError('Invalid File "{}": {}'.format(
                xml_file_path,
                str(exc),
            ))
        # check for XML syntax errors
        except etree.XMLSyntaxError as exc:
            self._logger.error(exc)
            raise StateLoadError('XML Syntax Error in "{}": {}'.format(
                xml_file_path,
                str(exc.error_log),
            ))
        except Exception as exc:
            self._logger.error(exc)
            raise StateLoadError('Unknown error in "{}": {}'.format(
                xml_file_path,
                str(exc),
            ))

        # validate against schema
        try:
            xmlschema.assertValid(doc)
        except etree.DocumentInvalid as exc:
            self._logger.error(exc)
            raise StateLoadError(
                'Schema validation error in "{}": {}'.format(
                    xml_file_path,
                    str(exc),
                )
            )
        except Exception as exc:
            self._logger.error(exc)
            raise StateLoadError(
                'Unknown validation error in "{}": {}'.format(
                    xml_file_path,
                    str(exc),
                )
            )

        return doc
