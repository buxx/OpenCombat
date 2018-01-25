# coding: utf-8
import typing

from opencombat.simulation.tmx import TileMap
from opencombat.simulation.tmx import TerrainTile
from synergine2_xyz.physics import MoveCostComputer
from synergine2_xyz.physics import TMXPhysics
from synergine2_xyz.subjects import XYZSubject

if typing.TYPE_CHECKING:
    from opencombat.simulation.base import BaseSubject
    from opencombat.simulation.subject import TileSubject


class TileMoveCostComputer(MoveCostComputer):
    def compute_move_cost(
        self,
        subject: 'BaseSubject',
        tile: TerrainTile,
        previous_node: str,
        next_node: str,
        unknown,
    ) -> float:
        # TODO #MoveCost: Write move cost system

        if not tile.property('traversable_by_man'):
            return 100

        return 1.0


class TilePhysics(TMXPhysics):
    tmx_map_class = TileMap
    move_cost_computer_class = TileMoveCostComputer
    matrixes_configuration = {
        'visibility': [
            'height',
            'opacity',
        ]
    }

    def get_visibility_obstacle(
        self,
        subject: XYZSubject,
        to_position: typing.Tuple[int, int],
        matrix_name: str,
        opacity_property_name: str='opacity',
    ) -> typing.Union[None, typing.Tuple[int, int]]:
        """
        Return grid position obstacle if any between given subject and given to_position
        :param subject: Subject observer
        :param to_position: position to observe
        :param matrix_name: matrix name to use
        :param opacity_property_name: name of property containing opacity property
        :return: obstacle grid position or None if not
        """
        from_x, from_y = subject.position
        path_positions = self.matrixes.get_path_positions(from_=(from_x, from_y), to=to_position)
        path_opacities = self.matrixes.get_values_for_path(
            matrix_name,
            path_positions=path_positions,
            value_name=opacity_property_name,
        )

        # TODO #Visibility: Write visibility system
        actual_opacity = 0
        for i, path_opacity in enumerate(path_opacities):
            actual_opacity += path_opacity
            if actual_opacity >= 100:
                return path_positions[i]

        return None

    def subject_see_subject(self, observer: 'TileSubject', observed: 'TileSubject') -> bool:
        return not bool(self.get_visibility_obstacle(
            observer,
            observed.position,
            matrix_name='visibility',
        ))
