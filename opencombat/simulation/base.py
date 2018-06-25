# coding: utf-8
from opencombat.const import COLLECTION_ALIVE
from opencombat.const import COUNTRY_USSR
from opencombat.const import FLAG_USSR
from opencombat.const import FLAG_DE
from opencombat.const import SIDE_ALLIES
from opencombat.const import SIDE_AXIS
from opencombat.const import COUNTRY_DE
from opencombat.const import USSR_COLOR
from opencombat.const import DE_COLOR
from opencombat.const import SIDE
from opencombat.const import FLAG
from opencombat.const import SELECTION_COLOR_RGB
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

    @classmethod
    def get_default_properties_for_country(cls, country: str) -> dict:
        if country == COUNTRY_USSR:
            return {
                SELECTION_COLOR_RGB: USSR_COLOR,
                FLAG: FLAG_USSR,
                SIDE: SIDE_ALLIES,
            }
        elif country == COUNTRY_DE:
            return {
                SELECTION_COLOR_RGB: DE_COLOR,
                FLAG: FLAG_DE,
                SIDE: SIDE_AXIS,
            }

        raise NotImplementedError('Unknown country "{}"'.format(country))

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
