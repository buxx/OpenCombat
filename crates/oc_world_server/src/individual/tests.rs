use glam::Vec2;
use oc_root::physics::Meters;
use std::assert_matches;

use oc_individual::{
    BodyGesture, Gesture, HandsGesture, Individual, IndividualIndex, Status, Weapon, Weapons,
    behavior::{Behavior, Intent},
    order::Order,
    squad::SquadIndex,
};
use oc_mod::{
    Mod,
    ammunition::AmmunitionIndex,
    grenade::{GrenadeIndex, GrenadeNature},
    magazine::MagazineIndex,
    weapons::WeaponIndex,
};
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    opacity::CumulatedOpacity,
    physics::Seconds,
    utils::{Frequency, U8Progress},
};
use oc_utils::d2::Direction;
use oc_world::{World, visibility::Visibility};

use crate::{
    index::Indexes,
    individual::{Processor, gesture::progress_, situation::Situation},
    runner::update::{ExplosionOrigin, Update},
};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    utils::workspace_root,
    weapons::{TestWeapon, TestWeapons},
    world::TestWorld,
};

const MOD: &str = "mods/tests1";
const FRAG_GRENADE: GrenadeIndex = GrenadeIndex(0);
const SMOKE_GRENADE: GrenadeIndex = GrenadeIndex(1);

#[test]
fn test_progress_from_zero_not_finished_1hz() {
    // Given
    let tick = Frequency::new(1.0);
    let progress = U8Progress(0);
    let total = Seconds(2.0);

    // When
    let (progress, lag) = progress_(tick, progress, total);

    // Then
    assert_eq!(progress, U8Progress(128));
    assert_eq!(lag, tick.interval());
}

#[test]
fn test_progress_from_half_finished_1hz() {
    // Given
    let tick = Frequency::new(1.0);
    let progress = U8Progress(128);
    let total = Seconds(2.0);

    // When
    let (progress, lag) = progress_(tick, progress, total);

    // Then
    assert_eq!(progress, U8Progress(255));
    assert_eq!(lag.as_micros(), 992187); // TODO: would like 1_000_000, but 255 ceil ... use something else than u8 ?
}

#[test]
fn test_progress_finished_with_exceedance_1hz() {
    // Given
    let tick = Frequency::new(1.0);
    let progress = U8Progress(254);
    let total = Seconds(2.0);

    // When
    let (progress, lag) = progress_(tick, progress, total);

    // Then
    assert_eq!(progress, U8Progress(255));
    assert_eq!(lag.as_micros(), 7812);
}

// Test orders distribution when squad own move to order
#[test]
fn test_distribute_move() {
    // Given
    let w = WorldConfig::new(100, 100)
        .with_formation_tiles_between_positions(2)
        .with_geo_pixels_per_tile(5);
    // test parameters (assume individual are all Idle in EST direction)
    let individual_1_position = WorldVec3::new(100., 100., 0.);
    let individual_2_position = WorldVec3::new(90., 110., 0.);
    let squad_position = WorldVec2::new(individual_1_position.x, individual_1_position.y);
    let move_to_position = WorldVec2::new(150., 100.);
    let move_to_order = Order::MoveTo(move_to_position);
    // expected
    let expected_individual_1_move_to_position = WorldVec2::new(150., 100.);
    let expected_individual_2_move_to_position = WorldVec2::new(150., 110.);

    let world = two_individuals_world(
        &w,
        individual_1_position,
        individual_2_position,
        squad_position,
        vec![move_to_order],
    );
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());

    // When
    let updates = processor.step();

    // Then
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(0),
        oc_individual::Update::SetOrders(vec![Order::MoveTo(
            expected_individual_1_move_to_position
        )])
    )));
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(1),
        oc_individual::Update::SetOrders(vec![Order::MoveTo(
            expected_individual_2_move_to_position
        )])
    )));
}

// Test orders distribution when squad have no order and squad member not at correct place
#[test]
fn test_distribute_idle() {
    // Given
    let w = WorldConfig::new(100, 100)
        .with_formation_tiles_between_positions(2)
        .with_geo_pixels_per_tile(5);
    // test parameters (assume individual are all Idle in EST direction)
    let individual_1_position = WorldVec3::new(100., 100., 0.);
    let individual_2_position = WorldVec3::new(90., 110., 0.);
    let squad_position = WorldVec2::new(individual_1_position.x, individual_1_position.y);
    // expected
    let expected_individual_2_move_to_position = WorldVec2::new(100., 110.);

    let world = two_individuals_world(
        &w,
        individual_1_position,
        individual_2_position,
        squad_position,
        vec![],
    );
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());

    // When
    let updates = processor.step();

    // Then
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(1),
        oc_individual::Update::SetOrders(vec![Order::MoveTo(
            expected_individual_2_move_to_position
        )])
    )),);
}

// Test update when individual have no order
#[test]
fn test_idle() {
    // Given
    let w = WorldConfig::new(100, 100)
        .with_formation_tiles_between_positions(2)
        .with_geo_pixels_per_tile(5);
    // test parameters (assume individual are all Idle in EST direction)
    let position = WorldVec3::new(100., 100., 0.);

    let world = one_individual_world(&w, position, vec![]);
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());

    // When
    let updates = processor.step();

    // Then
    assert_eq!(updates, vec![]);
}

// Test update when individual have idle order in other direction than current
#[test]
fn test_idle_order() {
    // Given
    let w = WorldConfig::new(100, 100)
        .with_formation_tiles_between_positions(2)
        .with_geo_pixels_per_tile(5);
    // test parameters (assume individual are all Idle in EST direction)
    let position = WorldVec3::new(100., 100., 0.);

    let mut world = one_individual_world(&w, position, vec![Order::Idle]);
    let index = Indexes::new(&world, &w);

    // When-Then
    {
        let processor = Processor::new(&world, &index, 0.into());
        let updates = processor.step();
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(0),
            oc_individual::Update::SetOrders(vec![Order::Idle])
        )));
    }

    // When-Then
    world.individuals[0].orders = vec![Order::Idle];
    let processor = Processor::new(&world, &index, 0.into());
    let updates = processor.step();
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(0),
        oc_individual::Update::Accomplished
    )));
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(0),
        oc_individual::Update::SetIntent(Intent::Idle(Direction::EST))
    )));
    assert!(updates.contains(&Update::UpdateSquad(
        SquadIndex(0),
        oc_individual::squad::Update::Accomplished
    )));
    assert!(updates.contains(&Update::UpdateSquad(
        SquadIndex(0),
        oc_individual::squad::Update::SetOrders(vec![])
    )));
}

fn engage_test_gesture(
    w: WorldConfig,
    setup_individual: impl FnOnce(&mut Individual),
) -> (Gesture, Vec<Update>) {
    // Given
    let position1 = WorldVec3::new(100., 100., 0.);
    let position2 = WorldVec3::new(100., 200., 0.);
    let mut world = one_vs_one_individual_world(
        &w,
        position1,
        position2,
        vec![Order::Idle],
        vec![Order::Idle],
    );
    setup_individual(world.individual_mut(IndividualIndex(0)));
    *world.visibilities.values_mut() = vec![
        vec![
            #[cfg(not(feature = "debug"))]
            Visibility::new(true, CumulatedOpacity(0.)),
            #[cfg(not(feature = "debug"))]
            Visibility::new(true, CumulatedOpacity(0.)),
            #[cfg(feature = "debug")]
            Visibility::new(true, CumulatedOpacity(0.), vec![]),
            #[cfg(feature = "debug")]
            Visibility::new(true, CumulatedOpacity(0.), vec![]),
        ],
        vec![
            #[cfg(not(feature = "debug"))]
            Visibility::new(true, CumulatedOpacity(0.)),
            #[cfg(not(feature = "debug"))]
            Visibility::new(true, CumulatedOpacity(0.)),
            #[cfg(feature = "debug")]
            Visibility::new(true, CumulatedOpacity(0.), vec![]),
            #[cfg(feature = "debug")]
            Visibility::new(true, CumulatedOpacity(0.), vec![]),
        ],
    ];
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());
    let individual1 = world.individual(IndividualIndex(0));
    let situation = processor.situation(individual1).clone();
    // When
    processor.engage_gesture(individual1, &situation, IndividualIndex(1))
}

#[test]
fn test_engage_begin() {
    // When-Then
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100);
    let gesture = engage_test_gesture(w, |individual1| {
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress::zero()));
    assert_eq!(gesture.1, vec![]);
}

#[test]
fn test_engage_aiming_progress() {
    // When-Then
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100)
        // 10 tick per seconds
        // (must be configured as it) 1.0 seconds to aim "Weapon1"
        // So, one tick -> 255 / 10 = 25 u8
        .with_individual_tick(Frequency::new(10.0));
    let gesture = engage_test_gesture(w, |individual1| {
        individual1.gesture = individual1
            .gesture
            .clone()
            .with_hands(HandsGesture::Aiming(U8Progress(0)));
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress(18)));
    assert_eq!(gesture.1, vec![]);
}

#[test]
fn test_engage_aiming_finished() {
    // When-Then
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100).with_individual_tick(Frequency::new(1.0));
    let gesture = engage_test_gesture(w, |individual1| {
        individual1.gesture = individual1
            .gesture
            .clone()
            .with_hands(HandsGesture::Aiming(U8Progress(254)));
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Idle);
    assert_eq!(gesture.1.len(), 2);
    assert_matches!(gesture.1[0], Update::SpawnProjectiles(_));
    let Update::SpawnProjectiles(_spawn) = &gesture.1[0] else {
        panic!("Spawn nature must be tested line before");
    };
    // assert_eq!(spawn.lag.as_micros(), 0); // FIXME BS NOW
    assert_eq!(
        gesture.1[1],
        Update::UpdateIndividual(
            IndividualIndex(0),
            oc_individual::Update::SetWeapons(Weapons {
                primary: Some(Weapon {
                    i: WeaponIndex(0),
                    filled: Some((MagazineIndex(0), AmmunitionIndex(0))),
                    filled_count: 4
                }),
                grenades: vec![],
            })
        )
    );
}

#[test]
fn test_engage_reload_finished() {
    // When-Then
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100);
    let gesture = engage_test_gesture(w, |individual1| {
        individual1.gesture = individual1
            .gesture
            .clone()
            .with_hands(HandsGesture::Reloading(U8Progress(254)));
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::not_filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
        individual1.magazines = vec![MagazineIndex(0)];
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress(0)));
    assert_eq!(
        gesture.1,
        vec![
            Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetWeapons(Weapons {
                    primary: Some(Weapon {
                        i: WeaponIndex(0),
                        filled: Some((MagazineIndex(0), AmmunitionIndex(0))),
                        filled_count: 5
                    }),
                    grenades: vec![],
                })
            ),
            Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetMagazines(vec![])
            )
        ]
    );
}

#[test]
fn test_engage_reload_start_with_compatible_magazine() {
    // Given
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100);

    // When
    let gesture = engage_test_gesture(w, |individual1| {
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::not_filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
        individual1.magazines = vec![MagazineIndex(1), MagazineIndex(0)];
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Reloading(U8Progress(0)));
}

#[test]
fn test_engage_reload_not_started_without_compatible_magazine() {
    // Given
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100);

    // When
    let gesture = engage_test_gesture(w, |individual1| {
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::not_filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
        individual1.magazines = vec![MagazineIndex(1)];
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Idle);
    assert_eq!(gesture.1, vec![]);
}

#[test]
fn test_engage_reload_finished_uses_compatible_magazine_only() {
    // Given
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100);

    // When
    let gesture = engage_test_gesture(w, |individual1| {
        individual1.gesture = individual1
            .gesture
            .clone()
            .with_hands(HandsGesture::Reloading(U8Progress(254)));
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::not_filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
        individual1.magazines = vec![MagazineIndex(1), MagazineIndex(0), MagazineIndex(1)];
    });

    // Then
    assert_eq!(gesture.0.hands, HandsGesture::Aiming(U8Progress(0)));
    assert_eq!(
        gesture.1,
        vec![
            Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetWeapons(Weapons {
                    primary: Some(Weapon {
                        i: WeaponIndex(0),
                        filled: Some((MagazineIndex(0), AmmunitionIndex(0))),
                        filled_count: 5
                    }),
                    grenades: vec![],
                })
            ),
            Update::UpdateIndividual(
                IndividualIndex(0),
                oc_individual::Update::SetMagazines(vec![MagazineIndex(1), MagazineIndex(1)])
            )
        ]
    );
}

#[test]
fn test_engage_reload_finished_without_compatible_magazine_changes_nothing() {
    // Given
    let mod_ = Mod::load(&workspace_root().join(MOD), None).unwrap();
    let w = WorldConfig::new(100, 100);

    // When
    let gesture = engage_test_gesture(w, |individual1| {
        individual1.gesture = individual1
            .gesture
            .clone()
            .with_hands(HandsGesture::Reloading(U8Progress(254)));
        *individual1 = individual1.clone().with_weapons(
            TestWeapons::builder()
                .primary(TestWeapon::not_filled(&mod_, "Weapon1").make())
                .build()
                .make(),
        );
        individual1.magazines = vec![MagazineIndex(1)];
    });

    // Then
    assert_eq!(gesture.1, vec![]);
}

// Refactored function which generate a world with one squad with two members.
// Both individuals Idle in EST direction.
fn two_individuals_world(
    w: &WorldConfig,
    individual_1_position: WorldVec3,
    individual_2_position: WorldVec3,
    squad_position: WorldVec2,
    squad_orders: Vec<Order>,
) -> World {
    let individual1 = TestIndividual::builder();
    let individual1 = individual1.position(individual_1_position);
    let individual1 = individual1
        .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
        .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
        .intent(Intent::Idle(Direction::EST));
    let individual1 = individual1.build().make(w);
    let individual2 = TestIndividual::builder();
    let individual2 = individual2.position(individual_2_position);
    let individual2 = individual2
        .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
        .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
        .intent(Intent::Idle(Direction::EST));
    let individual2 = individual2.build().make(w);

    let squad = TestSquad::builder();
    let squad = squad.position(squad_position);
    let squad = squad.members(vec![0.into(), 1.into()]);
    let squad = squad.orders(squad_orders);
    let squad = squad.build().make();

    let world = TestWorld::builder();
    let world = world.individuals(vec![individual1, individual2]);
    let world = world.squads(vec![squad]);

    world.build().make(w)
}

// Refactored function which generate a world with one squad with one member.
// Individuals Idle in EST direction.
fn one_individual_world(w: &WorldConfig, position: WorldVec3, orders: Vec<Order>) -> World {
    let individual = TestIndividual::builder();
    let individual = individual.position(position);
    let individual = individual
        .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
        .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
        .intent(Intent::Idle(Direction::EST));
    let individual = individual.build().make(w);

    let squad = TestSquad::builder();
    let squad = squad.position(WorldVec2::new(position.x, position.y));
    let squad = squad.members(vec![0.into()]);
    let squad = squad.orders(orders);
    let squad = squad.build().make();

    let world = TestWorld::builder();
    let world = world.individuals(vec![individual]);
    let world = world.squads(vec![squad]);

    world.build().make(w)
}

// Refactored function which generate a world with one squad with one member.
// Individuals Idle in EST direction.
fn one_vs_one_individual_world(
    w: &WorldConfig,
    position1: WorldVec3,
    position2: WorldVec3,
    orders1: Vec<Order>,
    orders2: Vec<Order>,
) -> World {
    let individual1 = TestIndividual::builder();
    let individual1 = individual1.position(position1);
    let individual1 = individual1
        .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
        .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
        .intent(Intent::Idle(Direction::EST));
    let individual1 = individual1.build().make(w);

    let squad1 = TestSquad::builder();
    let squad1 = squad1.position(WorldVec2::new(position1.x, position1.y));
    let squad1 = squad1.members(vec![0.into()]);
    let squad1 = squad1.orders(orders1);
    let squad1 = squad1.build().make();

    let individual2 = TestIndividual::builder();
    let individual2 = individual2.position(position2);
    let individual2 = individual2
        .gesture(Gesture::body(BodyGesture::StandUp(Direction::EST))) // Gesture & Behavior & Intent are important
        .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
        .intent(Intent::Idle(Direction::EST));
    let individual2 = individual2.build().make(w);

    let squad2 = TestSquad::builder();
    let squad2 = squad2.position(WorldVec2::new(position2.x, position2.y));
    let squad2 = squad2.members(vec![1.into()]);
    let squad2 = squad2.orders(orders2);
    let squad2 = squad2.build().make();

    let world = TestWorld::builder();
    let world = world.individuals(vec![individual1, individual2]);
    let world = world.squads(vec![squad1, squad2]);

    world.build().make(w)
}

#[cfg(not(feature = "debug"))]
fn visible_visibility() -> Visibility {
    Visibility::new(true, CumulatedOpacity(0.))
}

#[cfg(feature = "debug")]
fn visible_visibility() -> Visibility {
    Visibility::new(true, CumulatedOpacity(0.), vec![])
}

fn resolve_defend_order_result(
    initial_intent: Intent,
    target_status: Status,
    target_visible: bool,
) -> Intent {
    let w = WorldConfig::new(100, 100);
    let position1 = WorldVec3::new(100., 100., 0.);
    let position2 = WorldVec3::new(100., 200., 0.);
    let mut world = one_vs_one_individual_world(
        &w,
        position1,
        position2,
        vec![Order::Defend(Direction::EST)],
        vec![Order::Idle],
    );
    world.individual_mut(IndividualIndex(0)).intent = initial_intent;
    world.individual_mut(IndividualIndex(1)).status = target_status;

    let visibility = if target_visible {
        visible_visibility()
    } else {
        Visibility::default()
    };
    *world.visibilities.values_mut() = vec![
        // List of each individual visibility for individual 0
        vec![Visibility::default(), visibility.clone()],
        // List of each individual visibility for individual 1
        vec![visibility, Visibility::default()],
    ];

    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, IndividualIndex(0));
    let individual = world.individual(IndividualIndex(0));
    let situation = processor.situation(individual);

    processor.resolve_defend_order(individual, &situation, Direction::EST)
}

// Individual already engaging a target which is still visible and alive: keep engaging it.
#[test]
fn test_resolve_defend_order_continue_engage_when_target_visible_and_operational() {
    // Given-When
    let intent = resolve_defend_order_result(
        Intent::Engage(IndividualIndex(1)),
        Status::Operational,
        true,
    );

    // Then
    assert_eq!(intent, Intent::Engage(IndividualIndex(1)));
}

// Individual already engaging a target which is still visible but now dead: stop engaging it.
#[test]
fn test_resolve_defend_order_stop_engaging_when_target_visible_and_dead() {
    // Given-When
    let intent =
        resolve_defend_order_result(Intent::Engage(IndividualIndex(1)), Status::Dead, true);

    // Then
    assert_eq!(intent, Intent::Defend(Direction::EST));
}

// Individual already engaging a target which is still alive but no longer visible: stop engaging it.
#[test]
fn test_resolve_defend_order_stop_engaging_when_target_not_visible_and_operational() {
    // Given-When
    let intent = resolve_defend_order_result(
        Intent::Engage(IndividualIndex(1)),
        Status::Operational,
        false,
    );

    // Then
    assert_eq!(intent, Intent::Defend(Direction::EST));
}

// Individual not engaging anything, target becomes visible and alive: start engaging it.
#[test]
fn test_resolve_defend_order_start_engaging_when_target_newly_visible() {
    // Given-When
    let intent =
        resolve_defend_order_result(Intent::Defend(Direction::EST), Status::Operational, true);

    // Then
    assert_eq!(intent, Intent::Engage(IndividualIndex(1)));
}

// Individual not engaging anything, and no enemy visible: keep defending.
#[test]
fn test_resolve_defend_order_defend_when_no_enemy_visible() {
    // Given-When
    let intent =
        resolve_defend_order_result(Intent::Defend(Direction::EST), Status::Operational, false);

    // Then
    assert_eq!(intent, Intent::Defend(Direction::EST));
}

fn throw_test_world(w: &WorldConfig, order: Order, grenades: Vec<GrenadeIndex>) -> World {
    let mut world = one_individual_world(w, WorldVec3::new(100., 100., 0.), vec![Order::Idle]);
    let individual = world.individual_mut(IndividualIndex(0));
    individual.orders = vec![order];
    individual.weapons.grenades = grenades;
    world
}

#[test]
fn test_grenade_begin_reloads() {
    // Given
    let w = WorldConfig::new(100, 100);
    let target = WorldVec2::new(100., 200.);
    let world = throw_test_world(&w, Order::Grenade(target), vec![FRAG_GRENADE]);
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());
    let individual = world.individual(IndividualIndex(0));

    // When
    let (gesture, updates) = processor.throw_gesture(individual, target, GrenadeNature::Frag);

    // Then
    assert_eq!(gesture.hands, HandsGesture::Reloading(U8Progress::zero()));
    assert_eq!(updates, vec![]);
}

#[test]
fn test_grenade_reload_finished_aims() {
    // Given
    let w = WorldConfig::new(100, 100).with_individual_tick(Frequency::new(1.0));
    let target = WorldVec2::new(100., 200.);
    let mut world = throw_test_world(&w, Order::Grenade(target), vec![FRAG_GRENADE]);
    world.individual_mut(IndividualIndex(0)).gesture =
        Gesture::body(BodyGesture::Prone(Direction::EST))
            .with_hands(HandsGesture::Reloading(U8Progress(254)));
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());
    let individual = world.individual(IndividualIndex(0));

    // When
    let (gesture, updates) = processor.throw_gesture(individual, target, GrenadeNature::Frag);

    // Then
    assert_eq!(gesture.hands, HandsGesture::Aiming(U8Progress::zero()));
    assert_eq!(updates, vec![]);
}

#[test]
fn test_grenade_aiming_finished_throws() {
    // Given
    let w = WorldConfig::new(100, 100).with_individual_tick(Frequency::new(1.0));
    let target = WorldVec2::new(100., 200.);
    let mut world = throw_test_world(
        &w,
        Order::Grenade(target),
        vec![SMOKE_GRENADE, FRAG_GRENADE],
    );
    world.individual_mut(IndividualIndex(0)).gesture =
        Gesture::body(BodyGesture::Prone(Direction::EST))
            .with_hands(HandsGesture::Aiming(U8Progress(254)));
    let index = Indexes::new(&world, &w);
    let processor = Processor::new(&world, &index, 0.into());
    let individual = world.individual(IndividualIndex(0));

    // When
    let (gesture, updates) = processor.throw_gesture(individual, target, GrenadeNature::Frag);

    // Then
    assert_eq!(gesture.hands, HandsGesture::Idle);
    // The explosion grenade is consumed, not the smoke one
    let weapons = Weapons {
        primary: None,
        grenades: vec![SMOKE_GRENADE],
    };
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(0),
        oc_individual::Update::SetWeapons(weapons)
    )));
    assert!(updates.iter().any(|u| matches!(
        u,
        Update::Schedule(_, explosion)
            if matches!(**explosion, Update::SpawnExplosion(_, position, ExplosionOrigin::Grenade(FRAG_GRENADE)) if Vec2::from(position).distance(Vec2::from(target)) <= Meters(10.0).pixels(&w))
    )));
    // Order is finished
    assert!(updates.contains(&Update::UpdateIndividual(
        IndividualIndex(0),
        oc_individual::Update::Accomplished
    )));
}

#[test]
fn test_grenade_order_skipped_without_matching_grenade() {
    // Given
    let w = WorldConfig::new(100, 100);
    let target = WorldVec2::new(100., 200.);

    for (order, grenades) in [
        (Order::Grenade(target), vec![]),
        (Order::Grenade(target), vec![SMOKE_GRENADE]),
        (Order::Smoke(target), vec![FRAG_GRENADE]),
    ] {
        let world = throw_test_world(&w, order, grenades);
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.accomplished(&Situation { visibles: vec![] });

        // Then
        let updates = updates.expect("Order must be skipped");
        assert!(updates.contains(&Update::UpdateIndividual(
            IndividualIndex(0),
            oc_individual::Update::Accomplished
        )));
    }
}

#[test]
fn test_grenade_order_kept_with_matching_grenade() {
    // Given
    let w = WorldConfig::new(100, 100);
    let target = WorldVec2::new(100., 200.);

    for (order, grenades) in [
        (Order::Grenade(target), vec![FRAG_GRENADE]),
        (Order::Smoke(target), vec![SMOKE_GRENADE]),
    ] {
        let world = throw_test_world(&w, order, grenades);
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When-Then
        assert_eq!(
            processor.accomplished(&Situation { visibles: vec![] }),
            None
        );
    }
}
