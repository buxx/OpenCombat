# coding: utf-8
import typing

from synergine2.config import Config

from opencombat.const import SIDE
from opencombat.simulation.base import TileStrategySimulation

if typing.TYPE_CHECKING:
    from opencombat.simulation.subject import TileSubject


class Placement(object):
    """
    Place troops on a map
    """
    def __init__(
        self,
        config: Config,
        simulation: TileStrategySimulation,
    ) -> None:
        self._config = config
        self._simulation = simulation

    def place(self) -> None:
        # For now it is an extremely simple way to do it
        subject_by_sides = {}  # type: typing.Dict[str, typing.List[TileSubject]]  # nopep8
        for subject in self._simulation.subjects:
            subject_by_sides.setdefault(subject.properties[SIDE], []).append(
                subject,
            )

        x, y = 0, 0
        for side, subjects in subject_by_sides.items():
            y += 2
            for subject in subjects:
                x += 2

                subject.position = (x, y)
