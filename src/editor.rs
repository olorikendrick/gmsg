use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyCode, KeyModifiers, read},
};
use ratatui_textarea::TextArea;
pub struct Editor<'a> {
    state: State,
    textarea: TextArea<'a>,
}
#[derive(Debug, Default)]
enum State {
    #[default]
    Editing,
    Saved,
    Discarded,
}

impl<'a> Editor<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<String> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = read()? {
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        self.state = State::Saved;
                        break;
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                        self.state = State::Discarded;
                        return Ok(String::new());
                    }
                    _ => {
                        self.textarea.input(key);
                    }
                }
            }
        }
        let mut out = String::new();
        for line in self.textarea.lines() {
            out.push_str(&format!("{}\n", line));
        }
        Ok(out)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let width = area.width.max(3) - 3;

        frame.render_widget(&self.textarea, area);
    }
}

impl<'a> From<String> for Editor<'a> {
    fn from(input: String) -> Self {
        Self {
            state: State::Editing,
            textarea: TextArea::from(input.split("\n")),
        }
    }
}

impl<'a> Default for Editor<'a> {
    fn default() -> Self {
        Self {
            textarea: TextArea::default(),
            state: State::default(),
        }
    }
}
