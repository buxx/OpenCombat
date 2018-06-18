# coding: utf-8
import importlib

from _elementtree import Element
import xml.dom.minidom as md
from io import StringIO

from opencombat.exception import NotFoundError

__DEFAULT__ = '__DEFAULT__'


def get_class_from_string_path(config, string_path: str) -> type:
    """
    Return class matching with given path, eg "mymodule.MyClass"
    :param config: config object
    :param string_path: class path, eg "mymodule.MyClass"
    :return: imported class
    """
    module_address = '.'.join(string_path.split('.')[:-1])
    class_name = string_path.split('.')[-1]
    module_ = importlib.import_module(module_address)
    return getattr(module_, class_name)


def get_text_xml_element(
    element: Element,
    search_element_name: str,
    default_value: str = __DEFAULT__,
) -> str:
    found = element.find(search_element_name)
    if found is None:
        if default_value == __DEFAULT__:
            raise NotFoundError(
                'Asked element "{}" not exist in {}'.format(
                    search_element_name,
                    str(element),
                ),
            )
        return default_value

    return found.text


def pretty_xml(xml_str):
    """
    Return a pretty xmlstr of given xml str. Thank's to:
        https://gist.github.com/eliask/d8517790b11edac75983d1e6fdab3cab
    :param xml_str: ugly xml str
    :return: pretty xml str
    """
    indent = ' ' * 4
    return '\n'.join(
        line for line in
        md.parse(StringIO(xml_str)).toprettyxml(indent=indent).split('\n')
        if line.strip()
    )
