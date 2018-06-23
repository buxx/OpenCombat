# coding: utf-8
import os
import typing
from tkinter import Tk
from tkinter import Button
from tkinter import YES
from tkinter import StringVar
from tkinter import OptionMenu
from tkinter import W
from tkinter import E
from tkinter import messagebox
from tkinter.ttk import Combobox
from tkinter.ttk import Treeview


import time
from synergine2.config import Config

from opencombat.gui import Gui
from opencombat.strategy.manager import TroopManager
from opencombat.strategy.team.stash import TeamStash


class SelectTroopsGui(Gui):
    def __init__(
        self,
        config: Config,
        master: Tk,
        team_stash: TeamStash,
        troop_manager: TroopManager,
        countries: typing.List[str],
        troops_dir_path: str = '.',
    ) -> None:
        super().__init__(config, master)
        self._master.title('Troops selection')
        self._countries = countries
        self._team_stash = team_stash
        self._troop_manager = troop_manager
        self._troops_dir_path = troops_dir_path
        self._countries_troops = {}  # type: typing.Dict[str, typing.List[TeamModel]]  # nopep8

        # Widgets
        self._selected_country_var = StringVar(self._master)
        self._selected_country_var.set(countries[0])
        self._selected_country_var.trace('w', self._country_changed)
        self._select_country_menu = OptionMenu(
            self._master,
            self._selected_country_var,
            *countries,
        )

        self._teams_var = StringVar(self._master)
        self._teams_list = Combobox(
            self._master,
            height=10,
            state='readonly',
            textvariable=self._teams_var,
        )

        self._add_troop_var = StringVar(self._master)
        self._add_troop_button = Button(
            self._master,
            textvariable=self._add_troop_var,
            command=self._add_troop,
        )
        self._add_troop_var.set('Add troop')

        self._remove_troop_var = StringVar(self._master)
        self._remove_troop_button = Button(
            self._master,
            textvariable=self._remove_troop_var,
            command=self._remove_troop,
        )
        self._remove_troop_var.set('Remove troop')

        self._troops_view = Treeview(
            self._master,
            columns=('Soldiers',),
            height=10,
        )
        self._troops_view.heading('#0', text='Team')
        self._troops_view.heading('#1', text='Soldiers')
        self._troops_view.column('#0', stretch=YES)
        self._troops_view.column('#1', stretch=YES)

        self._generate_troops_var = StringVar(self._master)
        self._generate_troops_button = Button(
            self._master,
            textvariable=self._generate_troops_var,
            command=self._generate_troops,
        )
        self._generate_troops_var.set('Generate troops')

        # Layout
        self._select_country_menu.grid(row=0, column=0, sticky=W)
        self._teams_list.grid(row=1, column=0, sticky=W)
        self._add_troop_button.grid(row=2, column=0, sticky=W)
        self._troops_view.grid(row=3, column=0, sticky=W)
        self._remove_troop_button.grid(row=4, column=0, sticky=W)
        self._generate_troops_button.grid(row=4, column=0, sticky=E)

        # Default behaviours
        self._selected_country_var.set(countries[0])
        self._country_changed()

    def _country_changed(self, *args, **kwargs) -> None:
        country = self._selected_country_var.get()

        self._logger.info('Change country to "{}"'.format(
            country,
        ))
        country_team_names = [
            t.name for
            t in self._team_stash.get_team_by_country(
                self._selected_country_var.get(),
            )
        ]

        self._logger.debug('Change teams for: "{}"'.format(country_team_names))
        self._teams_list['values'] = country_team_names
        self._teams_var.set(country_team_names[0])
        self._update_troops_view(country)

    def _add_troop(self, *args, **kwargs) -> None:
        if self._teams_var.get():
            country = self._selected_country_var.get()
            team_name = self._teams_var.get()

            self._logger.info('Add troop "{}" to country "{}" troops'.format(
                team_name,
                team_name,
            ))

            team_model = self._team_stash.get_team_by_name(
                team_name=team_name,
                team_country=country,
            )
            self._countries_troops.setdefault(country, []).append(
                team_model,
            )
            self._update_troops_view(country)

    def _remove_troop(self, *args, **kwargs) -> None:
        selecteds = self._troops_view.selection()

        for selected in selecteds:
            team_name = self._troops_view.item(selected)['text']
            country = self._selected_country_var.get()

            self._logger.info('Remove team "{}" from country "{}"'.format(
                team_name,
                country,
            ))

            team_model = self._team_stash.get_team_by_name(
                team_name=team_name,
                team_country=country,
            )

            self._countries_troops[country].remove(team_model)

        if selecteds:
            self._update_troops_view(country)

    def _update_troops_view(self, country: str) -> None:
        teams = self._countries_troops.get(country, [])

        self._troops_view.delete(*self._troops_view.get_children())
        for team in teams:
            self._troops_view.insert(
                '',
                'end',
                text=team.name,
                values=('o' * len(team.units,))
            )

    def _generate_troops(self, *args, **kwargs) -> None:
        # Must have team(s) in all countries
        if len(self._countries_troops.keys()) == len(self._countries) \
                and all(self._countries_troops.values()):

            troops_file_path = os.path.join(
                self._troops_dir_path,
                'troops_{}.xml'.format(str(time.time())),
            )

            self._logger.info('Generate troops into file "{}"'.format(
                troops_file_path,
            ))

            troops_xml = self._troop_manager.get_troop_dump(
                self._countries_troops,
            )
            with open(troops_file_path, 'w+') as file:
                file.write(troops_xml)
        else:
            messagebox.showinfo(
                'Missing information',
                'All countries must have teams',
            )
