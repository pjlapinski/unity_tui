use super::Result;
use crate::{
    fs::ProjectFiles,
    ui::screen::{self, Screen},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::stdout, time::Duration};
use tui::backend::CrosstermBackend;
use tui::{backend::Backend, Frame, Terminal};

pub struct AppState {
    pub project: ProjectFiles,
    pub active_screen: Screen,
}

impl AppState {
    pub fn new(project: ProjectFiles) -> Self {
        let active_screen = Screen::new_file_select(&project);
        Self {
            project,
            active_screen,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match self.active_screen {
            Screen::FileSelect(..) => screen::file_select::handle_event(event, self),
        }
    }
}

pub fn run(project: ProjectFiles) -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = stdout();

    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let state = AppState::new(project);
    run_app(&mut terminal, state)?;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut state: AppState) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        if event::poll(Duration::from_millis(1))? {
            let event = event::read()?;
            match event {
                Event::Key(e) => match e {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        state: _,
                    } => break,
                    KeyEvent {
                        kind: KeyEventKind::Press,
                        ..
                    } => state.handle_event(&event),
                    _ => {}
                },
                Event::Paste(_) => {
                    state.handle_event(&event);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    match &mut state.active_screen {
        Screen::FileSelect(..) => screen::file_select::ui(f, state),
    }
}
