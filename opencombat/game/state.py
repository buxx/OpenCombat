# coding: utf-8

from synergine2.terminals import TerminalPackage
from synergine2_cocos2d.interaction import Interaction

from opencombat.simulation.state import SaveStateSimulationAction
from opencombat.user_action import UserAction


class SaveStateInteraction(Interaction):
    gui_action = UserAction.SAVE_STATE

    def get_package_for_terminal(self) -> TerminalPackage:
        return TerminalPackage(
            simulation_actions=[
                (SaveStateSimulationAction, {}),
            ]
        )
