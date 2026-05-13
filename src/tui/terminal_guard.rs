use ratatui::DefaultTerminal;

pub struct TerminalGuard(pub DefaultTerminal);
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
