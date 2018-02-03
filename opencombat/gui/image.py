# coding: utf-8
import io
import typing

from PIL import Image
from synergine2.config import Config
from synergine2_cocos2d.actor import Actor
from synergine2_cocos2d.util import PathManager

from opencombat.exception import UnknownAnimationIndex


class ImageCache(object):
    def __init__(self) -> None:
        self.cache = {}


class FiringCache(ImageCache):
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
            raise Exception('TODO')
        except IndexError:
            raise UnknownAnimationIndex(
                'Unknown animation index "{}" for mode "{}" and weapon "{}"'.format(
                    position,
                    mode,
                    weapon,
                ),
            )


class ImageCache(object):
    # FIXME: Move into synergine
    def __init__(
        self,
        actor: Actor,
        config: Config,
    ) -> None:
        self.config = config
        self.actor = actor

    def build(self) -> None:
        pass


class TileImageCache(ImageCache):
    def __init__(
        self,
        actor: Actor,
        config: Config,
    ) -> None:
        super().__init__(actor, config)
        self.firing_cache = FiringCache()
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
            mode_image_path = self.actor.default_image_path  # FIXME !
            mode_image = Image.open(self.path_manager.path(mode_image_path))

            for weapon in self.actor.weapons:
                images = self.actor.weapon_image_applier.get_firing_image(
                    mode=mode,
                    weapon_type=weapon,
                )

                for position in range(len(images)):
                    position_image = images[position]

                    final_image = mode_image.copy()
                    final_image.paste(
                        position_image,
                        (0, 0),
                        position_image,
                    )

                    self.firing_cache.add(mode, weapon, final_image)
