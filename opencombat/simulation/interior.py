# coding: utf-8
import typing

from PIL.PngImagePlugin import PngImageFile
from synergine2_xyz.map import TMXMap
from synergine2_xyz.utils import get_direct_around_positions_of_position


class InteriorMapConfiguration(object):
    def __init__(
        self,
        layer_name: str='interiors',
        exterior_id: str='ext',
        interior_id: str='int',
        separator_id: str='sep',
    ) -> None:
        self.layer_name = layer_name
        self.exterior_id = exterior_id
        self.interior_id = interior_id
        self.separator_id = separator_id


class InteriorManager(object):
    def __init__(
        self,
        map_: TMXMap,
        configuration: InteriorMapConfiguration=None,
    ) -> None:
        self.interiors = []
        self.map = map_
        self.configuration = configuration or InteriorMapConfiguration()
        self.interiors = self._compute_interiors()

    def _compute_interiors(self) -> typing.List[typing.List[typing.Tuple[int, int]]]:
        interiors = []
        layer_tiles = self.map.layer_tiles(self.configuration.layer_name)
        for tile_xy, tile in layer_tiles.items():

            # FIXME: on se retrouve avec des tuiles Grass la ou l'on a pas mis de tuile interior/exterior.
            # Faut pouvoir dire c'est tel tile la tile par defaut de tel layer
            if tile.property('id') == self.configuration.interior_id:
                x, y = map(int, tile_xy.split('.'))
                if not any([(x, y) in i for i in interiors]):
                    new_interior = [(x, y)]
                    positions_to_parse = []
                    possible_positions_xyz = get_direct_around_positions_of_position((x, y, 0))
                    possible_positions_xy = [(p[0], p[1]) for p in possible_positions_xyz]
                    positions_to_parse.extend(possible_positions_xy)

                    for possible_position_xyz in positions_to_parse:
                        test_tile = None
                        new_tile_x = possible_position_xyz[0]
                        new_tile_y = possible_position_xyz[1]
                        possible_position_key = '{}.{}'.format(new_tile_x, new_tile_y)

                        if (new_tile_x, new_tile_y) in new_interior:
                            continue

                        try:
                            test_tile = layer_tiles[possible_position_key]
                        except KeyError:
                            continue

                        if test_tile.property('id') not in [
                            self.configuration.interior_id,
                            self.configuration.separator_id,
                        ]:
                            continue

                        new_interior.append((new_tile_x, new_tile_y))

                        if not test_tile.property('id') == self.configuration.separator_id:
                            new_position_neighbour = get_direct_around_positions_of_position((new_tile_x, new_tile_y, 0))
                            positions_to_parse.extend(new_position_neighbour)

                    interiors.append(new_interior)

        return interiors

    def get_interiors(
        self,
        where_positions: typing.Iterable[typing.Tuple[int, int]]=None,
    ) -> typing.List[typing.List[typing.Tuple[int, int]]]:
        if where_positions is None:
            return self.interiors
        interiors = []

        for interior in self.interiors:
            for where_position in where_positions:
                if where_position in interior and interior not in interiors:
                    interiors.append(interior)
        return interiors

    def update_image_for_interiors(
        self,
        image: PngImageFile,
        interiors: typing.List[typing.List[typing.Tuple[int, int]]],
        tile_width: int,
        tile_height: int,
        invert_y: bool=True,
    ) -> None:
        # TODO BS 20171213: Optimization can be done: keep in cache modifications on image instead change it entirely
        pixels = image.load()

        for interior in interiors:
            for tile_x, tile_y in interior:
                start_x = tile_x * tile_width
                start_y = tile_y * tile_height
                for x in range(start_x, start_x+tile_width):
                    for y in range(start_y, start_y+tile_height):

                        real_y = y
                        if invert_y:
                            real_y = image.height - 1 - y

                        pixels[x, real_y] = (0, 0, 0, 0)
