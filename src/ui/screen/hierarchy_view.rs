use crate::unity::repository::MetaFilesRepository;
use crate::{
    fs,
    ui::{
        app::AppState,
        screen::{bordered_list, FooterRenderer, Screen, SelectNextPrev},
    },
    unity::{self, yaml},
    util::PairWith,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::{io::Error, path::PathBuf};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{List, ListItem, ListState},
    Frame,
};

use super::AvailableSize;

pub struct HierarchyViewState {
    pub selected_file_path: PathBuf,
    pub objects_repository: unity::Repository,
    pub game_objects_list_state: ListState,
    pub components_list_state: ListState,
    pub component_selected: bool,
    pub game_objects_list_len: usize,
    pub components_list_len: usize,
}

impl Screen {
    pub fn new_hierarchy_view(path: PathBuf) -> Result<Self, Error> {
        let repo = unity::construct_repository(yaml::parse_file(&path)?).unwrap();
        Ok(Screen::HierarchyView(HierarchyViewState {
            selected_file_path: path,
            objects_repository: repo,
            game_objects_list_state: ListState::default(),
            components_list_state: ListState::default(),
            component_selected: false,
            game_objects_list_len: 0,
            components_list_len: 0,
        }))
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let Screen::HierarchyView(view_state) = &mut state.active_screen else { unreachable!() };

    let size = f.get_available_size();

    let mut named_list = vec![];
    {
        let unparented = get_unparented(&view_state.objects_repository);
        for (game_object, transform) in unparented {
            named_list.append(
                &mut generate_game_object_named_list(
                    game_object,
                    transform,
                    0,
                    &view_state.objects_repository,
                )
                .unwrap(),
            );
        }
    }

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    {
        let t =
            fs::path_to_relative(&view_state.selected_file_path, &state.project.base_path).unwrap();
        let title = t.to_str().unwrap();
        let list = create_hierarchy_view(&named_list, title);

        if !named_list.is_empty() && view_state.game_objects_list_state.selected().is_none() {
            view_state.game_objects_list_state.select(Some(0));
        }
        view_state.game_objects_list_len = named_list.len();

        f.render_stateful_widget(list, layout[0], &mut view_state.game_objects_list_state);
    }

    {
        let selected = view_state.game_objects_list_state.selected().unwrap();
        let selected_game_object = named_list[selected].1;
        let list_items = get_components_list_items(
            &state.meta_files_repository,
            view_state,
            selected_game_object,
        );
        view_state.components_list_len = list_items.len();
        let list = bordered_list(list_items, Some(selected_game_object.name.clone()));

        f.render_stateful_widget(list, layout[1], &mut view_state.components_list_state);
    }

    if view_state.component_selected {
        f.render_footer("j/k/down/up: move  esc: hierarchy  ctrl+q: quit")
    } else {
        f.render_footer("j/k/down/up: move  space/enter: select  esc: select file  ctrl+q: quit")
    }
}

pub fn handle_event(event: &Event, state: &mut AppState) -> Result<(), Error> {
    let Screen::HierarchyView(view_state) = &mut state.active_screen else { unreachable!() };

    if let Event::Key(e) = event {
        match e {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if view_state.component_selected {
                    view_state.component_selected = false;
                    view_state.components_list_state.select(None);
                } else {
                    state.active_screen = Screen::new_file_select(&state.project);
                }
            }
            KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if view_state.component_selected {
                } else {
                    view_state.component_selected = true;
                    view_state.components_list_state.select(Some(0));
                }
            }
            KeyEvent {
                code: KeyCode::Char('j') | KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if view_state.component_selected {
                    view_state
                        .components_list_state
                        .next_if_some(view_state.components_list_len);
                } else {
                    view_state
                        .game_objects_list_state
                        .next_if_some(view_state.game_objects_list_len);
                }
            }
            KeyEvent {
                code: KeyCode::Char('k') | KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if view_state.component_selected {
                    view_state
                        .components_list_state
                        .prev_if_some(view_state.components_list_len);
                } else {
                    view_state
                        .game_objects_list_state
                        .prev_if_some(view_state.game_objects_list_len);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn generate_game_object_named_list<'a>(
    game_object: &'a unity::GameObject,
    transform: &unity::Transform,
    indent: usize,
    objects_repository: &'a unity::Repository,
) -> Option<Vec<(String, &'a unity::GameObject)>> {
    let mut out = vec![];
    out.push(if indent == 0 {
        (game_object.name.clone(), game_object)
    } else {
        (
            format!("{}└{}", " ".repeat(indent - 1), &game_object.name),
            game_object,
        )
    });

    let mut children = transform
        .get_children_ids()
        .iter()
        .map(|id| objects_repository.get_transform(id))
        .collect::<Option<Vec<&unity::Transform>>>()?;
    children.sort_by(|t1, t2| t1.partial_cmp_by_root_order(t2));

    for child in children {
        let go = objects_repository.get_game_object(child.get_game_object_id())?;
        out.append(&mut generate_game_object_named_list(
            go,
            child,
            indent + 1,
            objects_repository,
        )?);
    }

    Some(out)
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

fn create_hierarchy_view<'a>(
    game_object_named_list: &[(String, &unity::GameObject)],
    title: &'a str,
) -> List<'a> {
    let mut names = vec![];
    for (name, _go) in game_object_named_list.iter() {
        names.push(name);
    }
    let list_items: Vec<ListItem> = names
        .into_iter()
        .map(|name| ListItem::new(name.clone()).style(Style::reset()))
        .collect();

    bordered_list(list_items, Some(title))
}

fn get_components_list_items<'a>(
    meta_files_repository: &MetaFilesRepository,
    view_state: &HierarchyViewState,
    selected_game_object: &unity::GameObject,
) -> Vec<ListItem<'a>> {
    // let components: Option<Vec<&unity::Component>> = selected_game_object
    //     .component_ids
    //     .iter()
    //     .map(|id| objects_repository.get_component(id))
    //     .collect();
    // TODO: because some components are skipped for now, repository returns None for them. When this is fixed, replace next lines with the ones above
    let components: Option<Vec<&unity::Component>> = selected_game_object
        .component_ids
        .iter()
        .filter_map(|id| view_state.objects_repository.get_component(id))
        .map(Some)
        .collect();
    if let Some(components) = components {
        components
            .iter()
            .map(|comp| {
                let name = comp
                    .get_name(meta_files_repository)
                    .unwrap_or_else(|| "<Unrecognized Component>".to_owned()); // TODO: This should probably disappear when all components are implemented, look into it if not
                let mut enabled = true;
                if let unity::Component::MonoBehaviour(mono) = comp {
                    enabled = mono.enabled;
                }

                ListItem::new(name).style(if enabled {
                    Style::reset()
                } else {
                    Style::reset().fg(Color::Gray)
                })
            })
            .collect()
    } else {
        vec![]
    }
}
