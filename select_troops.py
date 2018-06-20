# coding: utf-8
import argparse
import typing
from tkinter import Tk

from synergine2.config import Config

from opencombat.strategy.manager import TroopManager
from opencombat.strategy.selection.gui import SelectTroopsGui


def main(
    units_file_path: str,
    teams_file_path: str,
    countries: typing.List[str],
) -> None:
    config = Config()
    config.load_yaml('config.yaml')

    troop_manager = TroopManager(
        config,
        units_file_path=units_file_path,
        teams_file_path=teams_file_path,
    )

    master = Tk()
    gui = SelectTroopsGui(
        config,
        master=master,
        troop_manager=troop_manager,
    )
    master.mainloop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Display troops selection gui',
    )
    parser.add_argument(
        '--units',
        dest='units_file_path',
        default='opencombat/strategy/units.xml',
    )
    parser.add_argument(
        '--teams',
        dest='teams_file_path',
        default='opencombat/strategy/teams.xml',
    )
    parser.add_argument(
        '--country',
        action='append',
        dest='countries',
        default=['URSS', 'DE'],
    )
    args = parser.parse_args()

    main(
        units_file_path=args.units_file_path,
        teams_file_path=args.teams_file_path,
        countries=args.countries,
    )
