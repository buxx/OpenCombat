use super::Panel;

pub struct DebugGuiState {
    pub panel: Panel,
}

impl Default for DebugGuiState {
    fn default() -> Self {
        Self {
            panel: Panel::default(),
        }
    }
}
