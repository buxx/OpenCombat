# coding: utf-8
import typing

from PIL import Image
from synergine2.config import Config
from synergine2_cocos2d.util import PathManager
from synergine2_xyz.image import ImageCache
from synergine2_xyz.image import ImageCacheManager
from synergine2_xyz.exception import UnknownAnimationIndex

from opencombat.exception import UnknownWeapon
from opencombat.exception import UnknownFiringAnimation

if typing.TYPE_CHECKING:
    from opencombat.gui.actor import BaseActor


class FiringImageCache(ImageCache):
    def add(
        self,
        mode: str,
        weapon: str,
        image: Image.Image,
    ) -> None:
        self.cache.setdefault(mode, {}).setdefault(weapon, []).append(image)

    def get(
        self,
        mode: str,
        weapon: str,
        position: int,
    ) -> Image.Image:
        try:
            return self.cache[mode][weapon][position]
        except KeyError:
            raise UnknownFiringAnimation(
                'Unknown firing animation for mode "{}" and weapon "{}"'.format(
                    mode,
                    weapon,
                )
            )
        except IndexError:
            raise UnknownAnimationIndex(
                'Unknown animation index "{}" for mode "{}" and weapon "{}"'.format(
                    position,
                    mode,
                    weapon,
                ),
            )

    def get_list(
        self,
        mode: str,
        weapon: str,
    ) -> typing.List[Image.Image]:
        try:
            return self.cache[mode][weapon]
        except KeyError:
            raise UnknownFiringAnimation(
                'Unknown firing animation for mode "{}" and weapon "{}"'.format(
                    mode,
                    weapon,
                )
            )


class TileImageCacheManager(ImageCacheManager):
    def __init__(
        self,
        actor: 'BaseActor',
        config: Config,
    ) -> None:
        super().__init__(actor, config)
        self.firing_cache = FiringImageCache()
        from opencombat.gui.actor import BaseActor
        self.actor = typing.cast(BaseActor, self.actor)
        self.path_manager = PathManager(
            self.config.resolve('global.include_path.graphics'),
        )

    def build(self) -> None:
        super().build()
        self.build_firing()

    def build_firing(self) -> None:
        for mode in self.actor.get_modes():
            mode_image_path = self.actor.get_mode_image_path(mode)
            mode_image = Image.open(self.path_manager.path(mode_image_path))

            for weapon in self.actor.weapons:
                try:
                    images = self.actor.weapon_image_applier.get_firing_image(
                        mode=mode,
                        weapon_type=weapon,
                    )
                except UnknownWeapon:
                    images = [Image.open(self.path_manager.path('empty.png'))]

                for position in range(len(images)):
                    position_image = images[position]

                    final_image = mode_image.copy()
                    final_image.paste(
                        position_image,
                        (0, 0),
                        position_image,
                    )

                    self.firing_cache.add(mode, weapon, final_image)
