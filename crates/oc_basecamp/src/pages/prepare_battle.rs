use eframe::egui;
use oc_root::side::Side;

use crate::{game, map_view, minimap, page::Page};

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some((battle, scenario)) = ctx.selected() else {
        ui.label("No scenario selected");
        return None;
    };
    let (a_name, b_name) = ctx
        .selected_mod_info()
        .map(|m| (m.a_name.as_str(), m.b_name.as_str()))?;

    map_view::button(
        ui,
        "prepare",
        &minimap::scenario_path(ctx.app_dir, battle, scenario, ctx.selected_mod),
    );
    ui.separator();

    let mut chosen = None;
    ui.columns(2, |columns| {
        for (side, name) in [(Side::A, a_name), (Side::B, b_name)] {
            let ui = &mut columns[side as usize];
            ui.heading(name);
            if ui.button(format!("Choose {name}")).clicked() {
                chosen = Some(side);
            }
        }
    });

    // If chosen, prepare battle files
    let side = chosen?;
    prepare_battle(ctx, battle, scenario, side)
}

fn prepare_battle(
    ctx: &mut PageContext<'_>,
    battle: &oc_battle::Battle,
    scenario: &oc_battle::Scenario,
    side: Side,
) -> Option<Page> {
    let result = ctx
        .selected_mod_info()
        .ok_or_else(|| format!("Mod '{}' not found", ctx.selected_mod))
        .and_then(|mod_| game::create(ctx.app_dir, battle, scenario, mod_, side))
        .and_then(|created| {
            ctx.games.insert(0, created.clone());
            game::open(ctx, scenario, created)
        });

    if let Err(error) = result {
        ctx.errors.push(format!("Start game: {error}"));
        *ctx.show_errors = true;
        return None;
    }

    Some(Page::ChooseSquads)
}
