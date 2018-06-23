# coding: utf-8
import typing

from opencombat.game.actor import BaseActor
from opencombat.simulation.event import DEFAULT_WEAPON_TYPE
from opencombat.simulation.fire import RequestFireBehaviour
from synergine2_cocos2d.interaction import BaseActorInteraction
from opencombat.user_action import UserAction
from synergine2.simulation import SimulationBehaviour
from synergine2_cocos2d.actor import Actor
from synergine2_cocos2d.gl import draw_line


class BaseFireActorInteraction(BaseActorInteraction):
    gui_action = None
    color = None
    not_visible_color = (0, 0, 0)
    request_move_behaviour_class = RequestFireBehaviour

    def draw_pending(self) -> None:
        for actor in self.layer_manager.edit_layer.selection:
            actor_grid_position = self.layer_manager.grid_manager.get_grid_position(actor.position)
            actor_pixel_position = self.layer_manager.grid_manager.get_world_position_of_grid_position(
                actor_grid_position,
            )
            mouse_grid_position = self.layer_manager.grid_manager.get_grid_position(
                self.layer_manager.scrolling_manager.screen_to_world(
                    *self.layer_manager.edit_layer.screen_mouse,
                )
            )
            draw_to_pixel = self.layer_manager.edit_layer.screen_mouse

            obstacle_grid_position = self.layer_manager.gui.physics.get_visibility_obstacle(
                subject=actor.subject,
                to_position=mouse_grid_position,
                matrix_name='visibility',
                opacity_property_name='opacity',
            )

            # DEBUG
            if self.layer_manager.debug:
                grid_paths = self.layer_manager.gui.physics.matrixes.get_path_positions(
                    from_=actor_grid_position,
                    to=mouse_grid_position,
                )
                previous_grid_path = None
                for grid_path in grid_paths:
                    if previous_grid_path:
                        previous_grid_path_pixel = self.layer_manager.grid_manager.get_world_position_of_grid_position(
                            previous_grid_path,
                        )
                        current_grid_pixel = self.layer_manager.grid_manager.get_world_position_of_grid_position(
                            grid_path,
                        )
                        draw_line(
                            self.layer_manager.scrolling_manager.world_to_screen(*previous_grid_path_pixel),
                            self.layer_manager.scrolling_manager.world_to_screen(*current_grid_pixel),
                            (25, 125, 25),
                        )
                    previous_grid_path = grid_path

            if obstacle_grid_position:
                obstacle_pixel = self.layer_manager.grid_manager.get_world_position_of_grid_position(
                    obstacle_grid_position,
                )
                draw_to_pixel = self.layer_manager.scrolling_manager.world_to_screen(*obstacle_pixel)

                draw_line(
                    self.layer_manager.scrolling_manager.world_to_screen(*obstacle_pixel),
                    self.layer_manager.edit_layer.screen_mouse,
                    self.not_visible_color,
                )

            draw_line(
                self.layer_manager.scrolling_manager.world_to_screen(*actor_pixel_position),
                draw_to_pixel,
                self.color,
            )

    def get_behaviour(self, actor: Actor, mouse_grid_position) -> typing.Tuple[typing.Type[SimulationBehaviour], dict]:
        raise NotImplementedError()
        return self.request_move_behaviour_class, {
            'subject_id': actor.subject.id,
            'move_to': mouse_grid_position,
            'gui_action': self.gui_action,
        }


class FireActorInteraction(BaseFireActorInteraction):
    gui_action = UserAction.ORDER_FIRE
    color = (255, 0, 0)


class GuiFiringEvent(object):
    def __init__(
        self,
        actor: BaseActor,
        weapon: str,
    ) -> None:
        self.actor = actor
        self.weapon = weapon
        self._animation_index = -1

        if weapon == DEFAULT_WEAPON_TYPE:
            self.weapon = self.actor.weapons[0]

    @property
    def animation_index(self) -> int:
        return self._animation_index

    def increment_animation_index(self) -> None:
        self._animation_index += 1

    def reset_animation_index(self) -> None:
        self._animation_index = -1
