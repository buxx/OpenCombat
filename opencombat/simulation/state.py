import os
import time
import typing

from synergine2.config import Config
from synergine2.simulation import SimulationBehaviour
from synergine2.simulation import Event
from synergine2.simulation import Simulation

from opencombat.state import StateConstructorBuilder


class SaveStateSimulationAction(SimulationBehaviour):
    def __init__(
        self,
        config: Config,
        simulation: Simulation,
    ):
        super().__init__(config, simulation)
        self.state_dumper = StateConstructorBuilder(
            config,
            simulation,
        ).get_state_dumper()
        self.state_save_dir = self.config.resolve('_runtime.state_save_dir')

    def run(self, data):
        pass

    def action(self, data) -> typing.List[Event]:
        state_file_path = os.path.join(
            self.state_save_dir,
            'state_{}.xml'.format(time.time())
        )
        with open(state_file_path, 'w+') as file:
            file.write(self.state_dumper.get_state_dump())

        return []

    @classmethod
    def merge_data(cls, new_data, start_data=None):
        pass