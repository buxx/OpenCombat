# coding: utf-8
import argparse
import logging
from random import seed

from synergine2.log import get_default_logger
from synergine2.config import Config
from synergine2_cocos2d.util import get_map_file_path_from_dir
from synergine2.core import Core
from synergine2.cycle import CycleManager
from synergine2.terminals import TerminalManager

from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.base import TileStrategySubjects
from opencombat.state import StateConstructorBuilder
from opencombat.terminal.base import CocosTerminal


def main(
    map_dir_path: str,
    seed_value: int=None,
    state_file_path: str=None,
    state_save_dir: str='.',
):
    if seed_value is not None:
        seed(seed_value)

    config = Config()
    config.load_yaml('config.yaml')

    # Runtime config
    config.setdefault('_runtime', {})['state_save_dir'] = state_save_dir

    level = logging.getLevelName(config.resolve('global.logging_level', 'ERROR'))
    logger = get_default_logger(level=level)

    map_file_path = get_map_file_path_from_dir(map_dir_path)

    simulation = TileStrategySimulation(config, map_file_path=map_file_path)
    subjects = TileStrategySubjects(simulation=simulation)

    if state_file_path:
        state_loader_builder = StateConstructorBuilder(config, simulation)
        state_loader = state_loader_builder.get_state_loader()
        state = state_loader.get_state(state_file_path)
        subjects.extend(state.subjects)

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
    parser.add_argument('--state', dest='state', default=None)
    parser.add_argument(
        '--state-save-dir',
        dest='state_save_dir',
        default='.',
    )

    args = parser.parse_args()

    main(
        args.map_dir_path,
        seed_value=args.seed,
        state_file_path=args.state,
        state_save_dir=args.state_save_dir,
    )
