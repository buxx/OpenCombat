# coding: utf-8
from synergine2.terminals import TerminalPackage
from synergine2_cocos2d.interaction import Interaction

from opencombat.simulation.placement import SetSubjectPositionsSimulationBehaviour  # nopep8
from opencombat.user_action import UserAction


class SetSubjectPositionsInteraction(Interaction):
    gui_action = UserAction.SET_SUBJECTS_POSITION

    def get_package_for_terminal(self) -> TerminalPackage:
        data = []  # type: typing.List[typing.Tuple[int, typing.Tuple[int, int]]]  # nopep8
        for moved_subject in self.layer_manager.edit_layer.selection.keys():
            grid_position = self.layer_manager.grid_manager.get_grid_position(
                moved_subject.position,
            )
            data.append(
                (moved_subject.subject.id, grid_position),
            )

        return TerminalPackage(
            simulation_actions=[
                (SetSubjectPositionsSimulationBehaviour, data),
            ]
        )
