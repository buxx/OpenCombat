# coding: utf-8
from opencombat.const import COLLECTION_ALIVE, COMBAT_MODE_DEFENSE
from opencombat.simulation.base import BaseSubject
from opencombat.simulation.behaviour import MoveToBehaviour
from opencombat.simulation.behaviour import EngageOpponent
from opencombat.simulation.behaviour import LookAroundBehaviour
from synergine2.share import shared


class TileSubject(BaseSubject):
    start_collections = [
        COLLECTION_ALIVE,
    ]
    behaviours_classes = [
        MoveToBehaviour,
        LookAroundBehaviour,
        EngageOpponent,
    ]
    visible_opponent_ids = shared.create_self('visible_opponent_ids', lambda: [])
    combat_mode = shared.create_self('combat_mode', COMBAT_MODE_DEFENSE)
    # TODO: implement (copied from engulf)
    # behaviour_selector_class = CellBehaviourSelector
