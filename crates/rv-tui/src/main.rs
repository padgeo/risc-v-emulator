//! Ratatui frontend for the RISC-V time-travel debugger.
//!
//! For now this only draws the planned pane layout. The panes are filled in as the emulator core
//! gains features (see `docs/roadmap.md`).

use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph},
};
use rv_core::memmap::RAM_BASE;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

/// UI state. Grows as panes are implemented.
#[derive(Debug, Default)]
struct App {
    should_quit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(draw)?;
            self.handle_event(&event::read()?);
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }
        // Raw mode turns Ctrl-C into an ordinary key event instead of a signal.
        let ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl_c || matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
            self.should_quit = true;
        }
    }
}

/// Disassembly on the left, registers, memory and console on the right, timeline along the bottom.
fn draw(frame: &mut Frame) {
    let [main, timeline] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());
    let [left, right] = Layout::horizontal([Constraint::Percentage(50); 2]).areas(main);
    let [registers, memory, console] = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Percentage(40),
        Constraint::Percentage(20),
    ])
    .areas(right);

    pane(frame, left, "Disassembly", "planned: roadmap session 7");
    pane(frame, registers, "Registers", "planned: roadmap session 7");
    pane(
        frame,
        memory,
        "Memory",
        &format!("RAM base {RAM_BASE:#010x}\nplanned: roadmap session 8"),
    );
    pane(frame, console, "Console", "planned: roadmap session 8");
    pane(
        frame,
        timeline,
        "Timeline",
        "planned: roadmap session 9 · q or Esc to quit",
    );
}

fn pane(frame: &mut Frame, area: Rect, title: &str, body: &str) {
    frame.render_widget(
        Paragraph::new(body).block(Block::bordered().title(title)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, crossterm::event::KeyEvent};

    use super::*;

    fn screen_text(buffer: &Buffer) -> String {
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn press(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    #[test]
    fn every_planned_pane_is_drawn() {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(draw).unwrap();
        let screen = screen_text(terminal.backend().buffer());
        for title in ["Disassembly", "Registers", "Memory", "Console", "Timeline"] {
            assert!(
                screen.contains(title),
                "missing pane {title:?} in:\n{screen}"
            );
        }
        assert!(
            screen.contains("0x80000000"),
            "RAM base missing in:\n{screen}"
        );
    }

    #[test]
    fn quit_keys_quit_and_other_keys_do_not() {
        for event in [
            press(KeyCode::Char('q'), KeyModifiers::NONE),
            press(KeyCode::Esc, KeyModifiers::NONE),
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            let mut app = App::default();
            app.handle_event(&event);
            assert!(app.should_quit, "{event:?} should quit");
        }

        let mut app = App::default();
        app.handle_event(&press(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&press(KeyCode::Char('c'), KeyModifiers::NONE));
        assert!(!app.should_quit);
    }
}
