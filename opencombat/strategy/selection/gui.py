# coding: utf-8
from tkinter import Tk
from tkinter import Label
from tkinter import W

from synergine2.config import Config

from opencombat.gui import Gui
from opencombat.strategy.manager import TroopManager


class SelectTroopsGui(Gui):
    def __init__(
        self,
        config: Config,
        master: Tk,
        troop_manager: TroopManager,
    ) -> None:
        super().__init__(config, master)
        self._master.title('Troops selection')
        self.label = Label(master, text="Hello")

        # Layout
        self.label.grid(row=0, column=0, sticky=W)
