use ratatui::DefaultTerminal;

pub struct TerminalGuard(pub DefaultTerminal);
impl Default for TerminalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalGuard {
    pub fn new() -> Self {
        Self(ratatui::init())
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
