use bevy::prelude::*;
use oc_individual::order::Order;
use oc_mod::illustration::Illustration;

use crate::sprites::{IntoIndividualSprite, IntoSprite, SpriteRect};

pub enum SquadOrderSprite {
    Idle,
    Move,
    MoveFast,
    Crawl,
    Engage,
    Suppress,
    Grenade,
    Smoke,
    Defend,
    Hide,
}

pub enum IndividualOrderSprite {
    Idle,
    Move,
    MoveFast,
    Crawl,
    Engage,
    Suppress,
    Grenade,
    Smoke,
    Defend,
    Hide,
}

impl IntoSprite<SquadOrderSprite> for Order {
    fn sprite(&self) -> SquadOrderSprite {
        match self {
            Order::Idle => SquadOrderSprite::Idle,
            Order::MoveTo(_) => SquadOrderSprite::Move,
            Order::MoveFastTo(_) => SquadOrderSprite::MoveFast,
            Order::SneakTo(_) => SquadOrderSprite::Crawl,
            Order::Defend(_) => SquadOrderSprite::Defend,
            Order::Hide(_) => SquadOrderSprite::Hide,
            Order::Engage(_) => SquadOrderSprite::Engage,
            Order::Suppress(_) => SquadOrderSprite::Suppress,
            Order::Grenade(_) => SquadOrderSprite::Grenade,
            Order::Smoke(_) => SquadOrderSprite::Smoke,
        }
    }
}

impl SpriteRect for SquadOrderSprite {
    fn rect(&self) -> Rect {
        const POSITION_START_X: f32 = 0.;
        const POSITION_START_Y: f32 = 145.;
        const POSITION_WIDTH: f32 = 11.;
        const POSITION_HEIGHT: f32 = 11.;

        const DIRECTION_START_X: f32 = 0.;
        const DIRECTION_START_Y: f32 = 190.;
        const DIRECTION_WIDTH: f32 = 50.;
        const DIRECTION_HEIGHT: f32 = 45.;

        match self {
            SquadOrderSprite::Idle => Rect::EMPTY, // Should never happen
            SquadOrderSprite::Move => {
                const INDEX: f32 = 0.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::MoveFast => {
                const INDEX: f32 = 1.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Crawl => {
                const INDEX: f32 = 2.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Engage => {
                const INDEX: f32 = 3.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Suppress => {
                const INDEX: f32 = 3.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X + POSITION_WIDTH,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Grenade => {
                const INDEX: f32 = 0.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X + POSITION_WIDTH,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Smoke => {
                const INDEX: f32 = 1.;
                let start_y = POSITION_START_Y + (INDEX * POSITION_HEIGHT);
                Rect::new(
                    POSITION_START_X + POSITION_WIDTH,
                    start_y,
                    POSITION_START_X + POSITION_WIDTH + POSITION_WIDTH,
                    start_y + POSITION_HEIGHT,
                )
            }
            SquadOrderSprite::Defend => {
                const INDEX: f32 = 1.;
                let start_y = DIRECTION_START_Y + (INDEX * DIRECTION_HEIGHT);
                Rect::new(
                    DIRECTION_START_X,
                    start_y,
                    DIRECTION_START_X + DIRECTION_WIDTH,
                    start_y + DIRECTION_HEIGHT,
                )
            }
            SquadOrderSprite::Hide => {
                const INDEX: f32 = 0.;
                let start_y = DIRECTION_START_Y + (INDEX * DIRECTION_HEIGHT);
                Rect::new(
                    DIRECTION_START_X,
                    start_y,
                    DIRECTION_START_X + DIRECTION_WIDTH,
                    start_y + DIRECTION_HEIGHT,
                )
            }
        }
    }
}

impl IntoIndividualSprite<IndividualOrderSprite> for Order {
    fn individual_sprite(&self) -> IndividualOrderSprite {
        match self {
            Order::Idle => IndividualOrderSprite::Idle,
            Order::MoveTo(_) => IndividualOrderSprite::Move,
            Order::MoveFastTo(_) => IndividualOrderSprite::MoveFast,
            Order::SneakTo(_) => IndividualOrderSprite::Crawl,
            Order::Defend(_) => IndividualOrderSprite::Defend,
            Order::Hide(_) => IndividualOrderSprite::Hide,
            Order::Engage(_) => IndividualOrderSprite::Engage,
            Order::Suppress(_) => IndividualOrderSprite::Suppress,
            Order::Grenade(_) => IndividualOrderSprite::Grenade,
            Order::Smoke(_) => IndividualOrderSprite::Smoke,
        }
    }
}

impl SpriteRect for IndividualOrderSprite {
    fn rect(&self) -> Rect {
        const START_X: f32 = 22.;
        const START_Y: f32 = 145.;
        const WIDTH: f32 = 11.;
        const HEIGHT: f32 = 11.;

        let (column, i) = match self {
            IndividualOrderSprite::Idle => return Rect::EMPTY, // Should never happen
            IndividualOrderSprite::Move => (0, 0),
            IndividualOrderSprite::MoveFast => (0, 1),
            IndividualOrderSprite::Crawl => (0, 2),
            IndividualOrderSprite::Engage => (0, 3),
            IndividualOrderSprite::Suppress => (0, 3),
            IndividualOrderSprite::Grenade => (1, 0),
            IndividualOrderSprite::Smoke => (1, 1),
            IndividualOrderSprite::Defend => return Rect::EMPTY, // Should never happen
            IndividualOrderSprite::Hide => return Rect::EMPTY,   // Should never happen
        };

        let start_x = START_X + (column as f32 * WIDTH);
        let start_y = START_Y + (i as f32 * HEIGHT);
        Rect::new(start_x, start_y, start_x + WIDTH, start_y + HEIGHT)
    }
}

impl SpriteRect for Illustration {
    fn rect(&self) -> Rect {
        let (x, y) = (self.x as f32, self.y as f32);
        Rect::new(x, y, x + self.width as f32, y + self.height as f32)
    }
}
