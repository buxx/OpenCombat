use ggez::graphics::{DrawParam, Rect};
use oc_core::{
    game::squad::SquadType,
    graphics::squad::{
        SQUAD_REL_TYPE1_HEIGHT, SQUAD_REL_TYPE1_START_X, SQUAD_REL_TYPE1_START_Y,
        SQUAD_REL_TYPE1_WIDTH, SQUAD_REL_TYPE_BREN_HEIGHT, SQUAD_REL_TYPE_BREN_START_X,
        SQUAD_REL_TYPE_BREN_START_Y, SQUAD_REL_TYPE_BREN_WIDTH, SQUAD_REL_TYPE_MG34_HEIGHT,
        SQUAD_REL_TYPE_MG34_START_X, SQUAD_REL_TYPE_MG34_START_Y, SQUAD_REL_TYPE_MG34_WIDTH,
    },
};

use super::utils::IntoDrawParam;

impl IntoDrawParam for SquadType {
    fn to_draw_param(&self) -> DrawParam {
        match self {
            SquadType::Type1 => DrawParam::new().src(Rect::new(
                SQUAD_REL_TYPE1_START_X,
                SQUAD_REL_TYPE1_START_Y,
                SQUAD_REL_TYPE1_WIDTH,
                SQUAD_REL_TYPE1_HEIGHT,
            )),
            SquadType::Bren => DrawParam::new().src(Rect::new(
                SQUAD_REL_TYPE_BREN_START_X,
                SQUAD_REL_TYPE_BREN_START_Y,
                SQUAD_REL_TYPE_BREN_WIDTH,
                SQUAD_REL_TYPE_BREN_HEIGHT,
            )),
            SquadType::Mg34 => DrawParam::new().src(Rect::new(
                SQUAD_REL_TYPE_MG34_START_X,
                SQUAD_REL_TYPE_MG34_START_Y,
                SQUAD_REL_TYPE_MG34_WIDTH,
                SQUAD_REL_TYPE_MG34_HEIGHT,
            )),
        }
    }
}
