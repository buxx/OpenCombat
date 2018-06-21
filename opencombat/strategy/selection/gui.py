# coding: utf-8
import typing
from tkinter import Tk
from tkinter import StringVar
from tkinter import OptionMenu
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
        countries: typing.List[str],
    ) -> None:
        super().__init__(config, master)
        self._master.title('Troops selection')

        # Widgets
        self.selected_country_var = StringVar(self._master)
        self.selected_country_var.set(countries[0])
        self.selected_country_var.trace('w', self.change_country)
        self.select_country_menu = OptionMenu(
            self._master,
            self.selected_country_var,
            *countries,
        )

        # Layout
        self.select_country_menu.grid(row=0, column=0, sticky=W)

    def change_country(self, *args, **kwargs) -> None:
        self._logger.info('Change country to "{}"'.format(
            self.selected_country_var.get(),
        ))
