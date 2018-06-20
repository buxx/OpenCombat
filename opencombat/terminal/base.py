# coding: utf-8
import pyglet
from cocos.director import event_loop
from synergine2.terminals import TerminalPackage

from opencombat.simulation.event import NewVisibleOpponent
from opencombat.simulation.event import FireEvent
from opencombat.simulation.event import DieEvent
from opencombat.simulation.event import NoLongerVisibleOpponent
from opencombat.simulation.physics import TilePhysics
from synergine2_cocos2d.terminal import GameTerminal
from synergine2_cocos2d.util import get_map_file_path_from_dir
from opencombat.simulation import move


class CocosTerminal(GameTerminal):
    main_process = True

    subscribed_events = [
        move.SubjectFinishTileMoveEvent,
        move.SubjectFinishMoveEvent,
        move.SubjectStartTileMoveEvent,
        move.SubjectStartRotationEvent,
        move.SubjectFinishRotationEvent,
        NewVisibleOpponent,
        NoLongerVisibleOpponent,
        FireEvent,
        DieEvent,
    ]

    def __init__(self, *args, asynchronous: bool, map_dir_path: str, **kwargs):
        super().__init__(*args, **kwargs)
        self.asynchronous = asynchronous
        map_file_path = get_map_file_path_from_dir(map_dir_path)
        self.physics = TilePhysics(
            self.config,
            map_file_path=map_file_path,
        )
        self.map_dir_path = map_dir_path

    def run(self):
        from opencombat.game.base import Game

        self.gui = Game(
            self.config,
            self,
            physics=self.physics,
            map_dir_path=self.map_dir_path,
        )

        @event_loop.event
        def on_window_close(window):
            event_loop.exit()
            self.send(TerminalPackage(sigterm=True))

            self.core_process.join(timeout=120)

            return pyglet.event.EVENT_HANDLED

        self.gui.run()
