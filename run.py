# coding: utf-8
import argparse
import logging
from random import seed

from synergine2.log import get_default_logger
from synergine2.config import Config
from synergine2_cocos2d.const import SELECTION_COLOR_RGB
from synergine2_cocos2d.util import get_map_file_path_from_dir
from synergine2.core import Core
from synergine2.cycle import CycleManager
from synergine2.terminals import TerminalManager

from opencombat.const import FLAG, SIDE
from opencombat.const import FLAG_DE
from opencombat.const import DE_COLOR
from opencombat.const import URSS_COLOR
from opencombat.const import FLAG_URSS
from opencombat.simulation.subject import ManSubject
from opencombat.simulation.subject import TankSubject
from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.base import TileStrategySubjects
from opencombat.state import StateLoaderBuilder
from opencombat.terminal.base import CocosTerminal


def main(
    map_dir_path: str,
    seed_value: int=None,
    state_file_path: str=None,
):
    if seed_value is not None:
        seed(seed_value)

    config = Config()
    config.load_yaml('config.yaml')
    level = logging.getLevelName(config.resolve('global.logging_level', 'ERROR'))
    logger = get_default_logger(level=level)

    map_file_path = get_map_file_path_from_dir(map_dir_path)

    simulation = TileStrategySimulation(config, map_file_path=map_file_path)
    subjects = TileStrategySubjects(simulation=simulation)

    if state_file_path:
        state_loader_builder = StateLoaderBuilder(config, simulation)
        state_loader = state_loader_builder.get_state_loader()
        state = state_loader.get_state(state_file_path)
        subjects.extend(state.subjects)

    # for position in ((10, 2), (11, 3), (11, 4), (12, 5),):
    #     man = ManSubject(
    #         config=config,
    #         simulation=simulation,
    #         position=position,
    #         properties={
    #             SELECTION_COLOR_RGB: DE_COLOR,
    #             FLAG: FLAG_DE,
    #             SIDE: 'AXIS',
    #         }
    #     )
    #     subjects.append(man)
    #
    # for position in ((30, 15), (31, 16), (32, 17), (33, 18),):
    #     man = ManSubject(
    #         config=config,
    #         simulation=simulation,
    #         position=position,
    #         properties={
    #             SELECTION_COLOR_RGB: URSS_COLOR,
    #             FLAG: FLAG_URSS,
    #             SIDE: 'ALLIES',
    #         }
    #     )
    #     subjects.append(man)
    #
    # for position in ((38, 24),):
    #     man = TankSubject(
    #         config=config,
    #         simulation=simulation,
    #         position=position,
    #         properties={
    #             SELECTION_COLOR_RGB: URSS_COLOR,
    #             FLAG: FLAG_URSS,
    #             SIDE: 'ALLIES',
    #         }
    #     )
    #     subjects.append(man)

    simulation.subjects = subjects

    core = Core(
        config=config,
        simulation=simulation,
        cycle_manager=CycleManager(
            config=config,
            simulation=simulation,
        ),
        terminal_manager=TerminalManager(
            config=config,
            terminals=[CocosTerminal(
                config,
                asynchronous=False,
                map_dir_path=map_dir_path,
            )]
        ),
        cycles_per_seconds=1 / config.resolve('core.cycle_duration'),
    )
    core.run()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Run TileStrategy')
    parser.add_argument('map_dir_path', help='map directory path')
    parser.add_argument('--seed', dest='seed', default=None)

    args = parser.parse_args()

    main(args.map_dir_path, seed_value=args.seed)
