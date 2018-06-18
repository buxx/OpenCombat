import typing

from synergine2.simulation import SimulationBehaviour, Event


class SaveStateSimulationAction(SimulationBehaviour):
    def run(self, data):
        pass

    def action(self, data) -> typing.List[Event]:
        # TODO BS 2018-06-14: dump state here
        pass

    @classmethod
    def merge_data(cls, new_data, start_data=None):
        pass