# coding: utf-8
import typing

from synergine2.config import Config
from synergine2.simulation import SimulationBehaviour
from synergine2.simulation import Simulation
from synergine2.simulation import Event
from synergine2.simulation import Intention
from synergine2_xyz.simulation import XYZSimulation


class FireIntention(Intention):
    def __init__(
        self,
        to_position: typing.Tuple[int, int],
        to_subject_id: int,
        gui_action: typing.Any,
    ) -> None:
        self.to_position = to_position
        self.to_subject_id = to_subject_id
        self.gui_action = gui_action


class RequestFireBehaviour(SimulationBehaviour):
    move_intention_class = FireIntention

    def __init__(
        self,
        config: Config,
        simulation: Simulation,
    ):
        super().__init__(config, simulation)
        self.simulation = typing.cast(XYZSimulation, self.simulation)

    def action(self, data) -> typing.List[Event]:
        to_position = data['to_position']
        to_subject_id = data['to_subject_id']

        return []
