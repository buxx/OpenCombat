# coding: utf-8
import typing

from synergine2.simulation import SimulationBehaviour
from synergine2.simulation import Event


class SetSubjectPositionsSimulationBehaviour(SimulationBehaviour):
    def action(self, data) -> typing.List[Event]:
        for subject_id, new_position in data:
            subject = self.simulation.subjects.index[subject_id]
            subject.position = new_position

        return []

    @classmethod
    def merge_data(cls, new_data, start_data=None):
        pass

    def run(self, data):
        pass
