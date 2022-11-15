use crate::unity::yaml;
use crate::util::PairWith;
use crate::{
    ui::{
        app::AppState,
        screen::{FooterRenderer, Screen},
    },
    unity,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::Error;
use std::path::PathBuf;
use tui::{backend::Backend, Frame};

pub struct FileViewState {
    pub selected_file_path: PathBuf,
    pub objects_repository: unity::Repository,
}

impl Screen {
    pub fn new_file_view(path: PathBuf) -> Result<Self, Error> {
        let repo = unity::construct_repository(yaml::parse_file(&path)?);
        Ok(Screen::FileView(FileViewState {
            selected_file_path: path,
            objects_repository: repo,
        }))
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let Screen::FileView(FileViewState {
        objects_repository,
        ..
    }) = &mut state.active_screen else {
        unreachable!()
    };

    let _unparented = objects_repository
        .get_unparented_transforms()
        .into_iter()
        .map(|trans| {
            objects_repository
                .get_game_object(trans.get_game_object_id())
                .unwrap()
                .pair_with(trans)
        })
        .collect::<Vec<(&unity::GameObject, &unity::Transform)>>();

    f.render_footer("esc: back  ctrl+q: quit")
}

pub fn handle_event(event: &Event, state: &mut AppState) -> Result<(), Error> {
    if let Event::Key(e) = event {
        #[allow(clippy::collapsible_match)]
        match e {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => state.active_screen = Screen::new_file_select(&state.project),
            _ => {}
        }
    }
    Ok(())
}
