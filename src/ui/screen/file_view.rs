use crate::ui::screen::bordered_list;
use crate::{
    fs,
    ui::{
        app::AppState,
        screen::{FooterRenderer, Screen},
    },
    unity::{self, yaml},
    util::PairWith,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::{io::Error, path::PathBuf};
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::AvailableSize;

pub struct FileViewState {
    pub selected_file_path: PathBuf,
    pub objects_repository: unity::Repository,
    pub game_objects_list_state: ListState,
}

impl Screen {
    pub fn new_file_view(path: PathBuf) -> Result<Self, Error> {
        let repo = unity::construct_repository(yaml::parse_file(&path)?).unwrap();
        Ok(Screen::FileView(FileViewState {
            selected_file_path: path,
            objects_repository: repo,
            game_objects_list_state: ListState::default(),
        }))
    }
}

fn generate_game_object_names(
    game_object: &unity::GameObject,
    transform: &unity::Transform,
    indent: usize,
    objects_repository: &unity::Repository,
) -> Option<Vec<String>> {
    let mut names = vec![];
    names.push(if indent == 0 {
        game_object.name.clone()
    } else {
        format!("{}└{}", " ".repeat(indent - 1), &game_object.name)
    });

    let mut children = transform
        .get_children_ids()
        .iter()
        .map(|id| objects_repository.get_transform(id))
        .collect::<Option<Vec<&unity::Transform>>>()?;
    children.sort_by(|t1, t2| t1.partial_cmp_by_root_order(t2));

    for child in children {
        let go = objects_repository.get_game_object(child.get_game_object_id())?;
        names.append(&mut generate_game_object_names(
            go,
            child,
            indent + 1,
            objects_repository,
        )?);
    }

    Some(names)
}

fn get_unparented(
    objects_repository: &unity::Repository,
) -> Vec<(&unity::GameObject, &unity::Transform)> {
    let mut sorted = objects_repository.get_unparented_transforms();
    sorted.sort_by(|t1, t2| t1.partial_cmp_by_root_order(t2));
    sorted
        .into_iter()
        .map(|trans| {
            objects_repository
                .get_game_object(trans.get_game_object_id())
                .unwrap()
                .pair_with(trans)
        })
        .collect()
}

pub fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let Screen::FileView(FileViewState {
                             selected_file_path,
                             objects_repository,
        game_objects_list_state,

    }) = &mut state.active_screen else {
        unreachable!()
    };

    let unparented = get_unparented(objects_repository);

    let mut names = vec![];
    for (go, trans) in unparented {
        names.append(&mut generate_game_object_names(go, trans, 0, objects_repository).unwrap());
    }
    let list_items: Vec<ListItem> = names
        .iter()
        .map(|name| ListItem::new(name.clone()).style(Style::reset()))
        .collect();

    if !list_items.is_empty() && game_objects_list_state.selected().is_none() {
        game_objects_list_state.select(Some(0));
    }
    let t = fs::path_to_relative(selected_file_path, &state.project.base_path).unwrap();
    let title = t.to_str().unwrap();

    let list = bordered_list(list_items, Some(title));

    f.render_stateful_widget(list, f.get_available_size(), game_objects_list_state);

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
