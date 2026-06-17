use crate::tui::TerminalGuard;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::fmt::Display;

pub struct Selector<T> {
    items: Vec<T>,
    state: ListState,
}

impl<T: Display + Clone> Selector<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { items, state }
    }

    pub fn run(&mut self, terminal: &mut TerminalGuard) -> anyhow::Result<Option<T>> {
        let terminal = &mut terminal.0;
        loop {
            terminal.draw(|f| self.render(f))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') => self.next(),
                    KeyCode::Up | KeyCode::Char('k') => self.previous(),
                    KeyCode::Enter => return Ok(self.selected()),
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                    _ => {}
                }
            }
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn selected(&self) -> Option<T> {
        self.state.selected().map(|i| self.items[i].clone())
    }

    fn render(&mut self, frame: &mut Frame) {
        let [header, list_area, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(
            Line::from(Span::styled(
                " Select an option",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            header,
        );

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|e| ListItem::new(format!("  {}", e)))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("❯ ");

        frame.render_stateful_widget(list, list_area, &mut self.state);

        frame.render_widget(
            Line::from(vec![
                Span::styled(" ↑↓ / jk", Style::default().fg(Color::DarkGray)),
                Span::raw("  navigate  "),
                Span::styled("Enter", Style::default().fg(Color::DarkGray)),
                Span::raw("  select  "),
                Span::styled("Esc/q", Style::default().fg(Color::DarkGray)),
                Span::raw("  cancel"),
            ]),
            footer,
        );
    }
}
