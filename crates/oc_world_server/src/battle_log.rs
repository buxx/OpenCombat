use oc_individual::{IndividualIndex, behavior::Behavior, squad::SquadIndex};
use oc_root::{geo::WorldVec2, side::Side};
use oc_utils::polygon::Polygon;
use oc_world::World;

/// Origin of a death
#[derive(Debug, Clone, Copy)]
pub enum Cause {
    /// Bullet shot by given individual (if known), from given position
    Gunshot(Option<IndividualIndex>, WorldVec2),
    /// Explosion kill rayon
    Explosion,
    /// Explosion shrapnel
    Shrapnel,
}

/// Squad of individual
pub fn squad_of(world: &World, i: IndividualIndex) -> Option<SquadIndex> {
    world
        .squads()
        .iter()
        .position(|s| s.members.contains(&i))
        .map(|s| SquadIndex(s as u64))
}

fn meters(world: &World, a: WorldVec2, b: WorldVec2) -> f32 {
    glam::Vec2::from(a).distance(b.into()) / world.w.geo_pixels_per_meters()
}

/// Log death of `victim` (call before applying death updates)
pub fn kill(world: &World, victim: IndividualIndex, cause: Cause) {
    let individual = world.individual(victim);
    if individual.status.is_dead() {
        return;
    }
    let position: WorldVec2 = individual.position.into();
    let interior = world.interiors.iter().any(|i| i.contains(position));
    let victim_squad = squad_of(world, victim);
    let victim_order = victim_squad.and_then(|s| world.squad(s).orders.first().cloned());

    let shooter = match cause {
        Cause::Gunshot(shooter, _) => shooter,
        _ => None,
    };
    let (cause_, shooter_squad, weapon, distance) = match cause {
        Cause::Gunshot(shooter, origin) => {
            let weapon = shooter.and_then(|s| {
                world
                    .individual(s)
                    .weapons
                    .primary
                    .as_ref()
                    .map(|w| world.mod_.weapon(w.i).inner().name().to_string())
            });
            (
                "gunshot",
                shooter.and_then(|s| squad_of(world, s)),
                weapon,
                Some(meters(world, origin, position)),
            )
        }
        Cause::Explosion => ("explosion", None, None, None),
        Cause::Shrapnel => ("shrapnel", None, None, None),
    };

    tracing::info!(
        name = "battle-kill",
        cause = cause_,
        victim_side = ?individual.side,
        victim = victim.0,
        victim_squad = ?victim_squad.map(|s| s.0),
        victim_order = ?victim_order,
        victim_body = ?individual.gesture.body,
        victim_interior = interior,
        shooter_squad = ?shooter_squad.map(|s| s.0),
        weapon = ?weapon,
        distance = ?distance.map(|d| d.round() as i32),
        shooter_suppress = ?shooter.map(|s| world.individual(s).suppress.0),
        shooter_body = ?shooter.map(|s| format!("{:?}", world.individual(s).gesture.body).split('(').next().unwrap_or_default().to_string()),
        victim_suppress = individual.suppress.0,
        shooter_behavior = ?shooter.map(|s| world.individual(s).action_hud_label()),
        victim_nature = nature_at(world, position),
        victim_behavior = individual.action_hud_label(),
    );
}

/// Log state of each squad of `side`
pub fn squads(world: &World, side: Side) {
    for (i, squad) in world.squads().iter().enumerate() {
        if squad.side != side {
            continue;
        }
        let leader = world.individual(squad.leader);
        let members = squad
            .members
            .iter()
            .map(|m| world.individual(*m))
            .filter(|m| m.status.is_target());
        let moving = members
            .clone()
            .filter(|m| {
                matches!(
                    m.behavior,
                    Behavior::Walk(_) | Behavior::Run(_) | Behavior::Crawl(_)
                )
            })
            .count();
        let firing = members
            .clone()
            .filter(|m| {
                matches!(
                    m.behavior,
                    Behavior::Engage(_)
                        | Behavior::Suppress(_)
                        | Behavior::Grenade(_)
                        | Behavior::Smoke(_)
                )
            })
            .count();
        tracing::info!(
            name = "battle-squads",
            side = ?side,
            squad = i,
            actives = squad.actives,
            members = squad.members.len(),
            x = leader.position.x.round() as i32,
            y = leader.position.y.round() as i32,
            moving,
            firing,
            elapsed = ?world.elapsed().map(|e| e.as_secs()),
            order = ?squad.orders.first(),
        );
    }
}

/// Log losses of both sides
pub fn summary(world: &World, label: &str) {
    let losses = |side: Side| {
        let individuals = world.individuals().iter().filter(|i| i.side == side);
        let total = individuals.clone().count();
        let dead = individuals.filter(|i| i.status.is_dead()).count();
        (dead, total)
    };
    let (a_dead, a_total) = losses(Side::A);
    let (b_dead, b_total) = losses(Side::B);
    let percent = |dead: usize, total: usize| match total {
        0 => 0,
        _ => (dead * 100 / total) as u32,
    };
    tracing::info!(
        name = "battle-summary",
        at = label,
        phase = ?world.phase,
        elapsed = ?world.elapsed().map(|e| e.as_secs()),
        flags = ?world.flags.iter().map(|(_, o)| o.clone()).collect::<Vec<_>>(),
        a_dead,
        a_total,
        a_loss_percent = percent(a_dead, a_total),
        b_dead,
        b_total,
        b_loss_percent = percent(b_dead, b_total),
    );
}

/// Nature name of the tile at `position`
fn nature_at(world: &World, position: WorldVec2) -> String {
    use oc_root::WcfgFrom;
    let xy = oc_geo::tile::TileXy::from_([position.x, position.y], &world.w);
    let index = oc_geo::tile::WorldTileIndex::from_(xy, &world.w);
    world
        .tile(index)
        .map(|t| world.mod_.nature(t.nature).name.clone())
        .unwrap_or_default()
}

/// Log position, behavior and order of each operational individual of `side`
pub fn members(world: &World, side: Side) {
    for (i, squad) in world.squads().iter().enumerate() {
        if squad.side != side {
            continue;
        }
        for member in &squad.members {
            let individual = world.individual(*member);
            if !individual.status.is_target() {
                continue;
            }
            let behavior = format!("{:?}", individual.behavior);
            let target = individual.orders.first().and_then(|o| o.position());
            tracing::info!(
                name = "battle-member",
                i = member.0,
                squad = i,
                leader = (*member == squad.leader),
                x = individual.position.x.round() as i32,
                y = individual.position.y.round() as i32,
                elapsed = ?world.elapsed().map(|e| e.as_millis()),
                behavior = behavior.split('(').next().unwrap_or_default(),
                order = ?individual.orders.first().map(|o| o.hud_label()),
                orders = individual.orders.len(),
                squad_order = ?squad.orders.first().map(|o| o.hud_label()),
                tx = ?target.map(|t| t.x.round() as i32),
                ty = ?target.map(|t| t.y.round() as i32),
                target_in_mesh = ?target.map(|t| world.navmesh.point_in_mesh((t.x, t.y))),
                in_mesh = world.navmesh.point_in_mesh((individual.position.x, individual.position.y)),
                path = ?individual.intent.path().map(|(_, p)| p.iter().count()),
                path_end = ?individual.intent.path().and_then(|(_, p)| p.iter().last().map(|e| (e.x.round() as i32, e.y.round() as i32))),
                suppress = individual.suppress.0,
            );
        }
    }
}
