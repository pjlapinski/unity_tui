use crate::util::hash_set;
use crate::{
    fs,
    ui::{
        app::AppState,
        screen::{bordered_list, FooterRenderer, Screen, SelectNextPrev},
    },
    unity::{self, repository::MetaFilesRepository, yaml},
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

pub enum HierarchyViewBlocksState {
    Hierarchy,
    GameObject,
    Component,
}

pub struct HierarchyViewState {
    pub selected_file_path: PathBuf,
    pub objects_repository: unity::Repository,
    pub game_objects_list_state: ListState,
    pub game_objects_list_len: usize,
    pub components_list_state: ListState,
    pub components_list_len: usize,
    pub fields_list_state: ListState,
    pub fields_list_len: usize,
    pub blocks_state: HierarchyViewBlocksState,
}

impl Screen {
    pub fn new_hierarchy_view(path: PathBuf) -> Result<Self, Error> {
        let repo = unity::construct_repository(yaml::parse_file(&path)?).unwrap();
        Ok(Screen::HierarchyView(HierarchyViewState {
            selected_file_path: path,
            objects_repository: repo,
            game_objects_list_state: ListState::default(),
            game_objects_list_len: 0,
            components_list_state: ListState::default(),
            components_list_len: 0,
            fields_list_state: ListState::default(),
            fields_list_len: 0,
            blocks_state: HierarchyViewBlocksState::Hierarchy,
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

    let t = fs::path_to_relative(&view_state.selected_file_path, &state.project.base_path).unwrap();
    let title = t.to_str().unwrap();
    let hierarchy_list = create_hierarchy_view(&named_list, title);
    if !named_list.is_empty() && view_state.game_objects_list_state.selected().is_none() {
        view_state.game_objects_list_state.select(Some(0));
    }
    view_state.game_objects_list_len = named_list.len();

    let selected_game_object_idx = view_state.game_objects_list_state.selected().unwrap();
    let selected_game_object = named_list[selected_game_object_idx].1;
    let components =
        get_game_object_components(&view_state.objects_repository, selected_game_object)
            .unwrap_or_default();
    let list_items = get_components_list_items(&state.meta_files_repository, &components);
    view_state.components_list_len = list_items.len();
    let components_list = bordered_list(list_items, Some(selected_game_object.name.clone()));

    let list_items: Vec<ListItem> =
        if let HierarchyViewBlocksState::Component = view_state.blocks_state {
            if view_state.fields_list_state.selected().is_none() {
                view_state.fields_list_state.select(Some(0));
            }
            if let Some(selected_component_idx) = view_state.components_list_state.selected() {
                let selected_component = components[selected_component_idx];
                let fields = get_components_fields(selected_component);
                fields
                    .iter()
                    .map(|field| ListItem::new(field.clone()).style(Style::reset()))
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
    view_state.fields_list_len = list_items.len();
    let title: Option<String> = None;
    let fields_list = bordered_list(list_items, title);

    match view_state.blocks_state {
        HierarchyViewBlocksState::Hierarchy => f.render_footer(
            "j/k/down/up: move  space/enter: select  esc: select file  ctrl+q: quit",
        ),
        HierarchyViewBlocksState::GameObject => {
            f.render_footer("j/k/down/up: move  esc: hierarchy  ctrl+q: quit")
        }
        HierarchyViewBlocksState::Component => {
            f.render_footer("j/k/down/up: move  esc: components  ctrl+q: quit")
        }
    }

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            match view_state.blocks_state {
                HierarchyViewBlocksState::GameObject | HierarchyViewBlocksState::Hierarchy => [
                    Constraint::Ratio(1, 2),
                    Constraint::Ratio(1, 2),
                    Constraint::Ratio(0, 2),
                ],
                HierarchyViewBlocksState::Component => [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ],
            }
            .as_ref(),
        )
        .split(size);

    f.render_stateful_widget(
        hierarchy_list,
        layout[0],
        &mut view_state.game_objects_list_state,
    );
    f.render_stateful_widget(
        components_list,
        layout[1],
        &mut view_state.components_list_state,
    );
    if let HierarchyViewBlocksState::Component = view_state.blocks_state {
        f.render_stateful_widget(fields_list, layout[2], &mut view_state.fields_list_state);
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
            } => match view_state.blocks_state {
                HierarchyViewBlocksState::Hierarchy => {
                    state.active_screen = Screen::new_file_select(&state.project);
                }
                HierarchyViewBlocksState::GameObject => {
                    view_state.blocks_state = HierarchyViewBlocksState::Hierarchy;
                    view_state.components_list_state.select(None);
                }
                HierarchyViewBlocksState::Component => {
                    view_state.blocks_state = HierarchyViewBlocksState::GameObject;
                    view_state.fields_list_state.select(None);
                }
            },
            KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => match view_state.blocks_state {
                HierarchyViewBlocksState::Hierarchy => {
                    view_state.blocks_state = HierarchyViewBlocksState::GameObject;
                    view_state.components_list_state.select(Some(0));
                }
                HierarchyViewBlocksState::GameObject => {
                    view_state.blocks_state = HierarchyViewBlocksState::Component;
                }
                HierarchyViewBlocksState::Component => {}
            },
            KeyEvent {
                code: KeyCode::Char('j') | KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => match view_state.blocks_state {
                HierarchyViewBlocksState::Hierarchy => {
                    view_state
                        .game_objects_list_state
                        .next_if_some(view_state.game_objects_list_len);
                }
                HierarchyViewBlocksState::GameObject => {
                    view_state
                        .components_list_state
                        .next_if_some(view_state.components_list_len);
                }
                HierarchyViewBlocksState::Component => {
                    view_state
                        .fields_list_state
                        .next_if_some(view_state.fields_list_len);
                }
            },
            KeyEvent {
                code: KeyCode::Char('k') | KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => match view_state.blocks_state {
                HierarchyViewBlocksState::Hierarchy => {
                    view_state
                        .game_objects_list_state
                        .prev_if_some(view_state.game_objects_list_len);
                }
                HierarchyViewBlocksState::GameObject => {
                    view_state
                        .components_list_state
                        .prev_if_some(view_state.components_list_len);
                }
                HierarchyViewBlocksState::Component => {
                    view_state
                        .fields_list_state
                        .prev_if_some(view_state.fields_list_len);
                }
            },
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
    let mut disabled_indices = hash_set![];
    for (idx, (name, go)) in game_object_named_list.iter().enumerate() {
        names.push(name);
        if !go.active {
            disabled_indices.insert(idx);
        }
    }
    let list_items: Vec<ListItem> = names
        .into_iter()
        .enumerate()
        .map(|(idx, name)| {
            if disabled_indices.contains(&idx) {
                ListItem::new(name.clone()).style(Style::reset().fg(Color::Gray))
            } else {
                ListItem::new(name.clone()).style(Style::reset())
            }
        })
        .collect();

    bordered_list(list_items, Some(title))
}

fn get_game_object_components<'a>(
    objects_repository: &'a unity::Repository,
    selected_game_object: &unity::GameObject,
) -> Option<Vec<&'a unity::Component>> {
    // let components: Option<Vec<&unity::Component>> = selected_game_object
    //     .component_ids
    //     .iter()
    //     .map(|id| objects_repository.get_component(id))
    //     .collect();
    // TODO: because some components are skipped for now, repository returns None for them. When this is fixed, replace next lines with the ones above
    selected_game_object
        .component_ids
        .iter()
        .filter_map(|id| objects_repository.get_component(id))
        .map(Some)
        .collect()
}

fn field_to_string(field: &unity::object::Field) -> String {
    match field {
        unity::object::Field::Vector2(vec) => format!("x:{} y:{}", vec.x, vec.y),
        unity::object::Field::Vector3(vec) => format!("x:{} y:{} z:{}", vec.x, vec.y, vec.z),
        unity::object::Field::Vector4(vec) => {
            format!("x:{} y:{} z:{} w:{}", vec.x, vec.y, vec.z, vec.w)
        }
        unity::object::Field::F64(f) => f.to_string(),
        unity::object::Field::I64(i) => i.to_string(),
        unity::object::Field::Str(s) => s.clone(),
        unity::object::Field::Bool(b) => b.to_string(),
        unity::object::Field::Yaml(_y) => "TEMPORARILY UNAVAILABLE".to_owned(),
    }
}

fn get_components_fields(selected_component: &unity::Component) -> Vec<String> {
    let mut out = vec![];
    match selected_component {
        unity::Component::MonoBehaviour(mono) => {
            out.push(format!("Enabled: {}", mono.enabled));
            for (name, field) in &mono.fields {
                out.push(format!(
                    "{}: {}",
                    unity::field_name_to_readable(name),
                    field_to_string(field)
                ));
            }
        }
        unity::Component::Transform(trans) => match trans {
            unity::Transform::Transform3D(trans) => {
                out.push(format!(
                    "Local Position: {}",
                    field_to_string(&unity::object::Field::Vector3(trans.local_position))
                ));
                out.push(format!(
                    "Local Rotation: {}",
                    field_to_string(&unity::object::Field::Vector4(trans.local_rotation))
                ));
                out.push(format!(
                    "Local Scale: {}",
                    field_to_string(&unity::object::Field::Vector3(trans.local_scale))
                ));
            }
            unity::Transform::RectTransform(trans) => {
                out.push(format!(
                    "Local Position: {}",
                    field_to_string(&unity::object::Field::Vector3(trans.local_position))
                ));
                out.push(format!(
                    "Local Rotation: {}",
                    field_to_string(&unity::object::Field::Vector4(trans.local_rotation))
                ));
                out.push(format!(
                    "Local Scale: {}",
                    field_to_string(&unity::object::Field::Vector3(trans.local_scale))
                ));
                out.push(format!(
                    "Pivot: {}",
                    field_to_string(&unity::object::Field::Vector2(trans.pivot))
                ));
                out.push(format!(
                    "Anchor Min: {}",
                    field_to_string(&unity::object::Field::Vector2(trans.anchor_min))
                ));
                out.push(format!(
                    "Anchor Max: {}",
                    field_to_string(&unity::object::Field::Vector2(trans.anchor_max))
                ));
                out.push(format!(
                    "Size Delta: {}",
                    field_to_string(&unity::object::Field::Vector2(trans.size_delta))
                ));
                out.push(format!(
                    "Anchored Position: {}",
                    field_to_string(&unity::object::Field::Vector2(trans.anchored_position))
                ));
            }
        },
    }
    out
}

fn get_components_list_items<'a>(
    meta_files_repository: &MetaFilesRepository,
    components: &[&unity::Component],
) -> Vec<ListItem<'a>> {
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
}
