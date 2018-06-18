# coding: utf-8
import io
import os
import random
import typing

import cocos
import pyglet
import time

from PIL import Image
from pyglet.window import key

from cocos.actions import MoveTo as BaseMoveTo
from cocos.actions import RotateTo
from synergine2_cocos2d.audio import AudioLibrary as BaseAudioLibrary
from synergine2_cocos2d.interaction import InteractionManager
from synergine2_cocos2d.middleware import MapMiddleware
from synergine2_cocos2d.util import PathManager

from opencombat.gui.animation import ANIMATION_WALK
from opencombat.gui.animation import ANIMATION_CRAWL
from opencombat.gui.fire import GuiFiringEvent
from opencombat.gui.state import SaveStateInteraction
from opencombat.simulation.interior import InteriorManager
from opencombat.simulation.tmx import TileMap
from opencombat.user_action import UserAction
from synergine2.config import Config
from synergine2.terminals import Terminal
from synergine2_cocos2d.actions import MoveTo
from synergine2_cocos2d.animation import Animate
from synergine2_cocos2d.gl import draw_line
from synergine2_cocos2d.gui import EditLayer as BaseEditLayer
from synergine2_cocos2d.gui import SubjectMapper
from synergine2_cocos2d.gui import Gui
from synergine2_cocos2d.gui import TMXGui
from synergine2_cocos2d.layer import LayerManager
from opencombat.simulation import move
from synergine2_xyz.physics import Physics
from synergine2_xyz.utils import get_angle
from opencombat.simulation.event import NewVisibleOpponent
from opencombat.simulation.event import NoLongerVisibleOpponent
from opencombat.simulation.event import FireEvent
from opencombat.simulation.event import DieEvent
from opencombat.simulation.subject import ManSubject
from opencombat.simulation.subject import TankSubject
from opencombat.gui.actor import Man as ManActor
from opencombat.gui.actor import HeavyVehicle as HeavyVehicleActor


class EditLayer(BaseEditLayer):
    def _on_key_press(self, k, m):
        if self.selection:
            if k == key.M:
                self.user_action_pending = UserAction.ORDER_MOVE
            if k == key.R:
                self.user_action_pending = UserAction.ORDER_MOVE_FAST
            if k == key.C:
                self.user_action_pending = UserAction.ORDER_MOVE_CRAWL
            if k == key.F:
                self.user_action_pending = UserAction.ORDER_FIRE

        if k == key.S:
            self.run_save_state()

    def draw(self) -> None:
        super().draw()

    def run_save_state(self) -> None:
        interaction = self.layer_manager\
            .interaction_manager\
            .get_for_user_action(
                UserAction.SAVE_STATE,
            )
        interaction.execute()


class BackgroundLayer(cocos.layer.Layer):
    def __init__(
        self,
        config: Config,
        layer_manager: LayerManager,
        background_sprite: cocos.sprite.Sprite,
    ) -> None:
        super().__init__()
        self.config = config
        self.layer_manager = layer_manager
        self.background_sprite = background_sprite
        self.last_interior_draw_timestamp = 0
        self.draw_interiors_gap = self.config.resolve(
            'game.building.draw_interior_gap',
            2,
        )
        self.background_image = Image.open(os.path.join(
            self.layer_manager.middleware.map_dir_path,
            'background.png',
        ))
        self.interior_manager = InteriorManager(
            TileMap(layer_manager.middleware.get_map_file_path()),
            original_image=self.background_image,
        )
        self.map_tile_width = self.layer_manager.middleware.get_cell_width()
        self.map_tile_height = self.layer_manager.middleware.get_cell_height()

    def draw(self, *args, **kwargs):
        super().draw(*args, **kwargs)
        self.draw_interiors()

    def draw_interiors(self):
        now = time.time()
        if now - self.last_interior_draw_timestamp > self.draw_interiors_gap:
            self.last_interior_draw_timestamp = now
            subject_grid_positions = [
                a.subject.position for a
                in self.layer_manager.subject_layer.subjects_index.values()
            ]
            interiors = self.interior_manager.get_interiors(
                where_positions=subject_grid_positions)

            # FIXME BS 2018-01-25: if not, put original background image
            if interiors:
                image_fake_file = io.BytesIO()
                new_background_image = self.interior_manager.update_image_for_interiors(
                    interiors,
                    self.map_tile_width,
                    self.map_tile_height,
                )
                new_background_image.save(image_fake_file, format='PNG')
                self.background_sprite.image = pyglet.image.load(
                    'new_background.png',
                    file=image_fake_file,
                )


class TileLayerManager(LayerManager):
    edit_layer_class = EditLayer

    def __init__(
        self,
        config: Config,
        middleware: MapMiddleware,
        interaction_manager: 'InteractionManager',
        gui: 'Gui',
    ) -> None:
        super().__init__(
            config,
            middleware,
            interaction_manager,
            gui,
        )
        self.background_layer = None  # type: BackgroundLayer
        self.interior_sprite = None  # type: cocos.sprite.Sprite
        self.ground_layer = None  # type: cocos.tiles.RectMapLayer
        self.top_layer = None  # type: cocos.tiles.RectMapLayer

    def init(self) -> None:
        super().init()
        self.interior_sprite = self.middleware.get_interior_sprite()
        background_sprite = self.middleware.get_background_sprite()
        self.background_layer = BackgroundLayer(self.config, self, background_sprite)
        self.background_layer.add(background_sprite)
        self.ground_layer = self.middleware.get_ground_layer()
        self.top_layer = self.middleware.get_top_layer()

    def connect_layers(self) -> None:
        self.main_layer.add(self.interior_sprite)
        self.main_layer.add(self.background_layer)
        self.main_layer.add(self.ground_layer)
        super().connect_layers()
        self.main_layer.add(self.top_layer)

    def center(self) -> None:
        super().center()
        self.interior_sprite.position = \
            0 + (self.interior_sprite.width / 2), 0 + (self.interior_sprite.height / 2)
        self.background_layer.background_sprite.position = \
            0 + (self.background_layer.background_sprite.width / 2), 0 +\
            (self.background_layer.background_sprite.height/2)
        self.ground_layer.set_view(
            0, 0, self.ground_layer.px_width, self.ground_layer.px_height,
        )
        # TODO BS 2018-06-14: We have to move this layer to be correct.
        # but some trees disapears ... wtf ?
        self.top_layer.set_view(
            28, 28, self.top_layer.px_width, self.top_layer.px_height,
        )


class AudioLibrary(BaseAudioLibrary):
    sound_file_paths = {
        'gunshot_default': '204010__duckduckpony__homemade-gunshot-2.ogg',
    }


class Game(TMXGui):
    layer_manager_class = TileLayerManager

    def __init__(
        self,
        config: Config,
        terminal: Terminal,
        physics: Physics,
        read_queue_interval: float = 1 / 60.0,
        map_dir_path: str=None,
    ):
        super().__init__(
            config,
            terminal,
            physics=physics,
            read_queue_interval=read_queue_interval,
            map_dir_path=map_dir_path,
        )
        self.sound_lib = AudioLibrary(config.resolve('global.include_path.sounds'))
        self.graphic_path_manager = PathManager(self.config.resolve(
            'global.include_path.graphics',
        ))
        self.debug_gui = self.config.resolve('global.debug_gui', False)

        self.terminal.register_event_handler(
            move.SubjectFinishTileMoveEvent,
            self.set_subject_position,
        )
        self.terminal.register_event_handler(
            move.SubjectFinishMoveEvent,
            self.set_subject_position,
        )

        self.terminal.register_event_handler(
            move.SubjectStartTileMoveEvent,
            self.start_move_subject,
        )

        self.terminal.register_event_handler(
            move.SubjectStartRotationEvent,
            self.start_rotate_subject,
        )

        self.terminal.register_event_handler(
            move.SubjectFinishRotationEvent,
            self.rotate_subject,
        )

        self.terminal.register_event_handler(
            NewVisibleOpponent,
            self.new_visible_opponent,
        )

        self.terminal.register_event_handler(
            NoLongerVisibleOpponent,
            self.no_longer_visible_opponent,
        )

        self.terminal.register_event_handler(
            FireEvent,
            self.fire_happen,
        )

        self.terminal.register_event_handler(
            DieEvent,
            self.subject_die,
        )

        self.dead_soldier_image = pyglet.resource.image(self.graphic_path_manager.path(
            'actors/man_d1.png',
        ))

        # subject/actor mapping
        self.subject_mapper_factory.register_mapper(
            ManSubject,
            SubjectMapper(self.config, ManActor),
        )
        self.subject_mapper_factory.register_mapper(
            TankSubject,
            SubjectMapper(self.config, HeavyVehicleActor),
        )

    def before_run(self) -> None:
        from opencombat.gui.move import MoveActorInteraction
        from opencombat.gui.move import MoveFastActorInteraction
        from opencombat.gui.move import MoveCrawlActorInteraction
        from opencombat.gui.fire import FireActorInteraction

        self.layer_manager.interaction_manager.register(MoveActorInteraction, self.layer_manager)
        self.layer_manager.interaction_manager.register(MoveFastActorInteraction, self.layer_manager)
        self.layer_manager.interaction_manager.register(MoveCrawlActorInteraction, self.layer_manager)
        self.layer_manager.interaction_manager.register(FireActorInteraction, self.layer_manager)
        self.layer_manager.interaction_manager.register(
            SaveStateInteraction,
            self.layer_manager,
        )

    def set_subject_position(
        self,
        event: typing.Union[move.SubjectFinishMoveEvent, move.SubjectFinishTileMoveEvent]
    ):
        actor = self.layer_manager.subject_layer.subjects_index[event.subject_id]
        new_world_position = self.layer_manager\
            .grid_manager\
            .get_world_position_of_grid_position(event.move_to)

        actor.stop_actions((BaseMoveTo,))
        actor.set_position(*new_world_position)

    def start_move_subject(
        self,
        event: typing.Union[move.SubjectFinishTileMoveEvent, move.SubjectContinueTileMoveEvent, move.SubjectFinishMoveEvent],  # nopep8
    ):
        actor = self.layer_manager.subject_layer.subjects_index[event.subject_id]
        new_world_position = self.layer_manager\
            .grid_manager\
            .get_world_position_of_grid_position(event.move_to)

        # FIXME BS 20180319: compute/config for cycle duration? ?
        if event.gui_action == UserAction.ORDER_MOVE:
            animation = ANIMATION_WALK
            cycle_duration = 2
        elif event.gui_action == UserAction.ORDER_MOVE_FAST:
            animation = ANIMATION_WALK
            cycle_duration = 0.5
        elif event.gui_action == UserAction.ORDER_MOVE_CRAWL:
            animation = ANIMATION_CRAWL
            cycle_duration = 2
        else:
            raise NotImplementedError(
                'Gui action {} unknown'.format(event.gui_action)
            )

        move_duration = event.duration
        move_action = MoveTo(new_world_position, move_duration)
        actor.do(move_action)
        actor.do(Animate(
            animation,
            duration=move_duration,
            cycle_duration=cycle_duration,
        ))
        if actor.can_rotate_instant():
            actor.rotation = get_angle(actor.subject.position, event.move_to)
        actor.mode = actor.get_mode_for_gui_action(animation)

    def start_rotate_subject(self, event: move.SubjectStartRotationEvent):
        actor = self.layer_manager.subject_layer.subjects_index[event.subject_id]
        rotate_action = RotateTo(event.rotate_absolute, event.duration)
        actor.stop_actions((RotateTo,))
        actor.do(rotate_action)

    def rotate_subject(self, event: move.SubjectFinishRotationEvent):
        actor = self.layer_manager.subject_layer.subjects_index[event.subject_id]
        actor.rotation = event.rotation_absolute

    def new_visible_opponent(self, event: NewVisibleOpponent):
        self.visible_or_no_longer_visible_opponent(event, (153, 0, 153))

    def no_longer_visible_opponent(self, event: NoLongerVisibleOpponent):
        self.visible_or_no_longer_visible_opponent(event, (255, 102, 0))

    def visible_or_no_longer_visible_opponent(
        self,
        event: typing.Union[NoLongerVisibleOpponent, NewVisibleOpponent],
        line_color,
    ) -> None:
        if not self.layer_manager.debug:
            return

        observer_actor = self.layer_manager.subject_layer.subjects_index[event.observer_subject_id]
        observed_actor = self.layer_manager.subject_layer.subjects_index[event.observed_subject_id]

        observer_pixel_position = self.layer_manager.scrolling_manager.world_to_screen(
            *self.layer_manager.grid_manager.get_world_position_of_grid_position(
                observer_actor.subject.position,
            )
        )
        observed_pixel_position = self.layer_manager.scrolling_manager.world_to_screen(
            *self.layer_manager.grid_manager.get_world_position_of_grid_position(
                observed_actor.subject.position,
            )
        )

        def draw_visible_opponent():
            draw_line(
                observer_pixel_position,
                observed_pixel_position,
                line_color,
            )

        self.layer_manager.edit_layer.append_callback(draw_visible_opponent, 1.0)

    def fire_happen(self, event: FireEvent) -> None:
        shooter_actor = self.layer_manager.subject_layer.subjects_index[event.shooter_subject_id]
        shooter_pixel_position = self.layer_manager.scrolling_manager.world_to_screen(
            *self.layer_manager.grid_manager.get_world_position_of_grid_position(
                shooter_actor.subject.position,
            )
        )
        fire_to_pixel_position = self.layer_manager.scrolling_manager.world_to_screen(
            *self.layer_manager.grid_manager.get_world_position_of_grid_position(
                event.target_position,
            )
        )

        def gunshot_trace():
            draw_line(
                shooter_pixel_position,
                fire_to_pixel_position,
                color=(255, 0, 0),
            )

        def gunshot_sound():
            self.sound_lib.get_sound('gunshot_default').play()

        firing_event = GuiFiringEvent(shooter_actor, event.weapon_type)

        def actor_rotate():
            shooter_actor.rotation = get_angle(
                shooter_actor.subject.position,
                event.target_position,
            )

        def actor_firing():
            shooter_actor.firing(firing_event)

        def actor_end_firing():
            shooter_actor.reset_default_texture()

        # To avoid all in same time
        # TODO BS 2018-01-24: This should be unecessary when core events sending will be
        # base on time base instead cycle base. Remove it to ensure.
        delay = random.uniform(0.0, 0.6)

        if self.debug_gui:
            self.layer_manager.edit_layer.append_callback(
                gunshot_trace,
                duration=0.1,
                delay=delay,
            )
        self.layer_manager.edit_layer.append_callback(
            gunshot_sound,
            duration=0.0,
            delay=delay,
        )
        self.layer_manager.edit_layer.append_callback(
            actor_firing,
            duration=0.5,  # TODO BS 2018-01-25: Wil depend of weapon type
            delay=delay,
            end_callback=actor_end_firing,
            start_callback=actor_rotate,
        )

    def subject_die(self, event: DieEvent) -> None:
        killed_actor = self.layer_manager.subject_layer.subjects_index[event.shoot_subject_id]
        killed_actor.update_image(self.dead_soldier_image)
        killed_actor.freeze()
