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

from opencc.const import FLAG, SIDE
from opencc.const import FLAG_DE
from opencc.const import DE_COLOR
from opencc.const import URSS_COLOR
from opencc.const import FLAG_URSS
from opencc.simulation.subject import TileSubject
from opencc.simulation.base import TileStrategySimulation
from opencc.simulation.base import TileStrategySubjects
from opencc.terminal.base import CocosTerminal


def main(map_dir_path: str, seed_value: int=None):
    if seed_value is not None:
        seed(seed_value)

    config = Config()
    config.load_yaml('config.yaml')
    logger = get_default_logger(level=logging.ERROR)

    map_file_path = get_map_file_path_from_dir(map_dir_path)

    simulation = TileStrategySimulation(config, map_file_path=map_file_path)
    subjects = TileStrategySubjects(simulation=simulation)

    for position in ((10, 2), (11, 3), (11, 4), (12, 5),):
        man = TileSubject(
            config=config,
            simulation=simulation,
            position=position,
            properties={
                SELECTION_COLOR_RGB: DE_COLOR,
                FLAG: FLAG_DE,
                SIDE: 'AXIS',
            }
        )
        subjects.append(man)

    for position in ((30, 15), (31, 16), (32, 17), (33, 18),):
        man = TileSubject(
            config=config,
            simulation=simulation,
            position=position,
            properties={
                SELECTION_COLOR_RGB: URSS_COLOR,
                FLAG: FLAG_URSS,
                SIDE: 'ALLIES',
            }
        )
        subjects.append(man)

    simulation.subjects = subjects

    core = Core(
        config=config,
        logger=logger,
        simulation=simulation,
        cycle_manager=CycleManager(
            config=config,
            logger=logger,
            simulation=simulation,
        ),
        terminal_manager=TerminalManager(
            config=config,
            logger=logger,
            terminals=[CocosTerminal(
                config,
                logger,
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
