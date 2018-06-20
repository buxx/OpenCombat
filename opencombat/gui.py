# coding: utf-8
from logging import Logger
from tkinter import Tk

from synergine2.config import Config
from synergine2.log import get_logger


class Gui(object):
    def __init__(
        self,
        config: Config,
        master: Tk,
    ) -> None:
        self._config = config
        self._logger = get_logger(self.__class__.__name__, config)
        self._master = master
