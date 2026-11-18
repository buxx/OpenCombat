use eframe::egui;
use oc_root::side::Side;

use crate::page::Page;

use super::PageContext;

const A_DEPLOYMENT_FILE: &str = "a_deployment.yml";
const B_DEPLOYMENT_FILE: &str = "b_deployment.yml";

/// Deployment file name (in game folder) of given side
pub fn deployment_file(side: Side) -> &'static str {
    match side {
        Side::A => A_DEPLOYMENT_FILE,
        Side::B => B_DEPLOYMENT_FILE,
    }
}

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some((battle, scenario)) = ctx.selected() else {
        ui.label("No scenario selected");
        return None;
    };

    ui.heading(format!("{}: {}", battle.name(), scenario.name()));
    ui.separator();
    ui.label("todo");
    ui.add_space(8.0);

    if ui.button("Start").clicked() {
        // Game is created when player choose its side
        *ctx.current_game = None;
        return Some(Page::PrepareBattle);
    }

    None
}
