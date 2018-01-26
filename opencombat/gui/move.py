# coding: utf-8
import typing

from synergine2_cocos2d.interaction import BaseActorInteraction
from opencombat.user_action import UserAction
from synergine2.simulation import SimulationBehaviour
from synergine2_cocos2d.actor import Actor
from synergine2_cocos2d.gl import draw_line
from synergine2_xyz.move.simulation import RequestMoveBehaviour


class BaseMoveActorInteraction(BaseActorInteraction):
    gui_action = None
    color = None
    request_move_behaviour_class = RequestMoveBehaviour

    def draw_pending(self) -> None:
        for actor in self.layer_manager.edit_layer.selection:
            grid_position = self.layer_manager\
                .grid_manager\
                .get_grid_position(actor.position)
            pixel_position = self.layer_manager\
                .grid_manager\
                .get_world_position_of_grid_position(grid_position)

            draw_line(
                self.layer_manager.scrolling_manager.world_to_screen(*pixel_position),
                self.layer_manager.edit_layer.screen_mouse,
                self.color,
            )

    def get_behaviour(
        self,
        actor: Actor,
        mouse_grid_position,
    ) -> typing.Tuple[typing.Type[SimulationBehaviour], dict]:
        return self.request_move_behaviour_class, {
            'subject_id': actor.subject.id,
            'move_to': mouse_grid_position,
            'gui_action': self.gui_action,
        }


class MoveActorInteraction(BaseMoveActorInteraction):
    gui_action = UserAction.ORDER_MOVE
    color = (0, 0, 255)


class MoveFastActorInteraction(BaseMoveActorInteraction):
    gui_action = UserAction.ORDER_MOVE_FAST
    color = (72, 244, 66)


class MoveCrawlActorInteraction(BaseMoveActorInteraction):
    gui_action = UserAction.ORDER_MOVE_CRAWL
    color = (235, 244, 66)
