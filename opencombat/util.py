# coding: utf-8
import importlib


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
