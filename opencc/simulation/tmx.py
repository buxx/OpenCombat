# coding: utf-8
import tmx

from synergine2_xyz.map import TMXMap
from synergine2_xyz.map import XYZTile


class TerrainTile(XYZTile):
    pass


class TileMap(TMXMap):
    xyz_tile_class = TerrainTile

    def get_default_tileset(self) -> tmx.Tileset:
        return self.tmx_tilesets['terrain']
