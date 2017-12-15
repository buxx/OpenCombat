# coding: utf-8
from opencombat.const import COLLECTION_ALIVE
from opencombat.simulation.physics import TilePhysics
from synergine2.config import Config
from synergine2.simulation import SubjectBehaviour
from synergine2_xyz.physics import Physics
from synergine2_xyz.simulation import XYZSimulation
from synergine2_xyz.subjects import XYZSubject
from synergine2_xyz.subjects import XYZSubjects


class TileStrategySimulation(XYZSimulation):
    behaviours_classes = [

    ]

    def __init__(
        self,
        config: Config,
        map_file_path: str,
    ) -> None:
        self.map_file_path = map_file_path
        super().__init__(config)

    def create_physics(self) -> Physics:
        return TilePhysics(
            config=self.config,
            map_file_path=self.map_file_path,
        )


class TileStrategySubjects(XYZSubjects):
    pass


class BaseSubject(XYZSubject):
    pass


class AliveSubjectBehaviour(SubjectBehaviour):
    def is_terminated(self) -> bool:
        return COLLECTION_ALIVE not in self.subject.collections
