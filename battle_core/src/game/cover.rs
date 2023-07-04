use crate::{
    config::{ServerConfig, COVER_DISTANCE},
    entity::soldier::Soldier,
    map::find_arbitrary_cover_grid_point,
    state::battle::BattleState,
    types::{GridPoint, SoldierIndex, SquadComposition, WorldPoint},
    utils::NewDebugPoint,
};

use super::squad::{squad_positions, Formation};

pub struct CoverFinder<'a> {
    battle_state: &'a BattleState,
    config: &'a ServerConfig,
    point: Option<WorldPoint>,
    exclude_grid_points: Vec<GridPoint>,
}

impl<'a> CoverFinder<'a> {
    pub fn new(battle_state: &'a BattleState, config: &'a ServerConfig) -> Self {
        Self {
            battle_state,
            config,
            point: None,
            exclude_grid_points: vec![],
        }
    }

    pub fn point(mut self, point: Option<WorldPoint>) -> Self {
        self.point = point;
        self
    }

    pub fn exclude_grid_points(mut self, points: Vec<GridPoint>) -> Self {
        self.exclude_grid_points = points;
        self
    }

    pub fn find_cover_points(
        &self,
        squad: &SquadComposition,
        leader: &Soldier,
    ) -> (
        Vec<(SoldierIndex, WorldPoint, WorldPoint)>,
        Vec<NewDebugPoint>,
    ) {
        let mut moves = vec![];
        let mut already_used_cover_grid_points: Vec<GridPoint> = self.exclude_grid_points.clone();
        let mut debug_points = vec![];

        for (member_id, formation_position) in
            squad_positions(squad, Formation::Line, leader, self.point)
        {
            let soldier = self.battle_state.soldier(member_id);
            let grid_point = self
                .battle_state
                .map()
                .grid_point_from_world_point(&formation_position);
            if let Some((cover_grid_point, debug_grid_points)) = find_arbitrary_cover_grid_point(
                &self.config,
                &grid_point,
                &self.battle_state.map(),
                &already_used_cover_grid_points,
                COVER_DISTANCE,
            ) {
                if self.config.send_debug_points {
                    for debug_grid_point in debug_grid_points.iter() {
                        debug_points.push(NewDebugPoint {
                            point: self
                                .battle_state
                                .map()
                                .world_point_from_grid_point(*debug_grid_point),
                        })
                    }
                }

                let from_world_point = soldier.get_world_point();
                let cover_world_point = self
                    .battle_state
                    .map()
                    .world_point_from_grid_point(cover_grid_point);
                already_used_cover_grid_points.push(cover_grid_point);

                moves.push((member_id, from_world_point, cover_world_point));
            }
        }

        (moves, debug_points)
    }
}
