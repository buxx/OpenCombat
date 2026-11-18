use crate::pages::PageContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Solo,
    Multiplayer,
    Campaign,
    Battle,
    NewBattle,
    PrepareBattle,
    ChooseSquads,
    BattleInProgress,
    BattleFinished,
}

impl Page {
    pub fn title(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Solo => "Solo",
            Page::Multiplayer => "Multiplayer",
            Page::Campaign => "Campaign",
            Page::Battle => "Battle",
            Page::NewBattle => "New Battle",
            Page::PrepareBattle => "Prepare Battle",
            Page::ChooseSquads => "Choose Squads",
            Page::BattleInProgress => "Battle in progress",
            Page::BattleFinished => "Battle finished",
        }
    }

    pub fn path(&self) -> Vec<Page> {
        match self {
            Page::Home => vec![Page::Home],
            Page::Solo | Page::Multiplayer => vec![Page::Home, *self],
            Page::Campaign | Page::Battle => vec![Page::Home, Page::Solo, *self],
            Page::NewBattle => vec![Page::Home, Page::Solo, Page::Battle, *self],
            Page::PrepareBattle => {
                vec![Page::Home, Page::Solo, Page::Battle, Page::NewBattle, *self]
            }
            Page::ChooseSquads | Page::BattleFinished => vec![
                Page::Home,
                Page::Solo,
                Page::Battle,
                Page::NewBattle,
                Page::PrepareBattle,
                *self,
            ],
            Page::BattleInProgress => vec![
                Page::Home,
                Page::Solo,
                Page::Battle,
                Page::NewBattle,
                Page::PrepareBattle,
                Page::ChooseSquads,
                *self,
            ],
        }
    }

    pub fn show(&self, ui: &mut eframe::egui::Ui, ctx: &mut PageContext) -> Option<Page> {
        use crate::pages;
        match self {
            Page::Home => pages::home::show(ui),
            Page::Solo => pages::solo::show(ui),
            Page::Multiplayer => pages::multiplayer::show(ui),
            Page::Campaign => pages::campaign::show(ui),
            Page::Battle => pages::battle::show(ui, ctx),
            Page::NewBattle => pages::new_battle::show(ui, ctx),
            Page::PrepareBattle => pages::prepare_battle::show(ui, ctx),
            Page::ChooseSquads => pages::choose_squads::show(ui, ctx),
            Page::BattleInProgress => pages::battle_in_progress::show(ui, ctx),
            Page::BattleFinished => pages::battle_finished::show(ui, ctx),
        }
    }
}
