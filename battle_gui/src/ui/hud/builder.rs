use battle_core::{
    state::battle::BattleState,
    types::{Scale, WindowPoint},
};
use ggez::{
    graphics::{DrawParam, Rect},
    Context,
};

use crate::{
    engine::state::GuiState,
    ui::{
        component::background::HorizontalBackground,
        hud::{
            BACKGROUND_CENTER_HEIGHT, BACKGROUND_CENTER_WIDTH, BACKGROUND_LEFT_WIDTH,
            BACKGROUND_REL_CENTER_HEIGHT, BACKGROUND_REL_CENTER_START_X,
            BACKGROUND_REL_CENTER_START_Y, BACKGROUND_REL_CENTER_WIDTH, BACKGROUND_RIGHT_HEIGHT,
            BACKGROUND_RIGHT_WIDTH,
        },
    },
};

use super::{
    background::Background, Hud, BACKGROUND_LEFT_HEIGHT, BACKGROUND_REL_LEFT_HEIGHT,
    BACKGROUND_REL_LEFT_START_X, BACKGROUND_REL_LEFT_START_Y, BACKGROUND_REL_LEFT_WIDTH,
    BACKGROUND_REL_RIGHT_HEIGHT, BACKGROUND_REL_RIGHT_START_X, BACKGROUND_REL_RIGHT_START_Y,
    BACKGROUND_REL_RIGHT_WIDTH, HUD_FACTOR, HUD_HEIGHT,
};

pub struct HudBuilder<'a> {
    gui_state: &'a GuiState,
    battle_state: &'a BattleState,
}

impl<'a> HudBuilder<'a> {
    pub fn new(gui_state: &'a GuiState, battle_state: &'a BattleState) -> Self {
        Self {
            gui_state,
            battle_state,
        }
    }

    pub fn build(&self, ctx: &Context) -> Hud {
        let window = ctx.gfx.window().inner_size();
        let background = HorizontalBackground {
            rel_left_start_x: BACKGROUND_REL_LEFT_START_X,
            rel_left_start_y: BACKGROUND_REL_LEFT_START_Y,
            rel_left_width: BACKGROUND_REL_LEFT_WIDTH,
            rel_left_height: BACKGROUND_REL_LEFT_HEIGHT,
            left_width: BACKGROUND_LEFT_WIDTH,
            left_height: BACKGROUND_LEFT_HEIGHT,
            rel_center_start_x: BACKGROUND_REL_CENTER_START_X,
            rel_center_start_y: BACKGROUND_REL_CENTER_START_Y,
            rel_center_width: BACKGROUND_REL_CENTER_WIDTH,
            rel_center_height: BACKGROUND_REL_CENTER_HEIGHT,
            center_width: BACKGROUND_CENTER_WIDTH,
            center_height: BACKGROUND_CENTER_HEIGHT,
            rel_right_start_x: BACKGROUND_REL_RIGHT_START_X,
            rel_right_start_y: BACKGROUND_REL_RIGHT_START_Y,
            rel_right_width: BACKGROUND_REL_RIGHT_WIDTH,
            rel_right_height: BACKGROUND_REL_RIGHT_HEIGHT,
            right_width: BACKGROUND_RIGHT_WIDTH,
            right_height: BACKGROUND_RIGHT_HEIGHT,
        };
        let background_sprites = background.sprites(
            WindowPoint::new(0., window.height as f32 - HUD_HEIGHT),
            window.width as f32,
            HUD_HEIGHT,
            HUD_FACTOR,
        );

        // println!("{}", (window.width as f32 / 2.) / BACKGROUND_REL_LEFT_WIDTH);
        // let background = Background::new(vec![
        //     DrawParam::new()
        //         .src(Rect::new(
        //             BACKGROUND_REL_LEFT_START_X,
        //             BACKGROUND_REL_LEFT_START_Y,
        //             BACKGROUND_REL_LEFT_WIDTH,
        //             BACKGROUND_REL_LEFT_HEIGHT,
        //         ))
        //         .scale(
        //             Scale::new(
        //                 (window.width as f32 / 2.) / BACKGROUND_LEFT_WIDTH,
        //                 HUD_HEIGHT / BACKGROUND_LEFT_HEIGHT,
        //             )
        //             .to_vec2(),
        //         )
        //         .dest(WindowPoint::new(0., window.height as f32 - HUD_HEIGHT).to_vec2()),
        //     DrawParam::new()
        //         .src(Rect::new(
        //             BACKGROUND_REL_RIGHT_START_X,
        //             BACKGROUND_REL_RIGHT_START_Y,
        //             BACKGROUND_REL_RIGHT_WIDTH,
        //             BACKGROUND_REL_RIGHT_HEIGHT,
        //         ))
        //         .scale(
        //             Scale::new(
        //                 (window.width as f32 / 2.) / BACKGROUND_RIGHT_WIDTH,
        //                 HUD_HEIGHT / BACKGROUND_RIGHT_HEIGHT,
        //             )
        //             .to_vec2(),
        //         )
        //         .dest(
        //             WindowPoint::new(window.width as f32 / 2.0, window.height as f32 - HUD_HEIGHT)
        //                 .to_vec2(),
        //         ),
        // ]);
        let background = Background::new(background_sprites);
        Hud::new(background)
    }
}
