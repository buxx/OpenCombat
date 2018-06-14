# coding: utf-8
import typing

from synergine2.simulation import SimulationBehaviour
from synergine2.simulation import Event
from synergine2.terminals import TerminalPackage
from synergine2_cocos2d.interaction import Interaction

from opencombat.user_action import UserAction


class SaveStateSimulationAction(SimulationBehaviour):
    def run(self, data):
        pass

    def action(self, data) -> typing.List[Event]:
        # TODO BS 2018-06-14: dump state here
        pass

    @classmethod
    def merge_data(cls, new_data, start_data=None):
        pass


class SaveStateInteraction(Interaction):
    gui_action = UserAction.SAVE_STATE

    def get_package_for_terminal(self) -> TerminalPackage:
        return TerminalPackage(
            simulation_actions=[
                (SaveStateSimulationAction, {}),
            ]
        )
