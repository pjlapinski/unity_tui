use crate::ui::screen::bordered_list;
use crate::{
    fs::{self, ProjectFiles},
    ui::{
        app::AppState,
        screen::{AvailableSize, FooterRenderer, Screen, SelectNextPrev},
    },
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::Error;
use std::path::{Path, PathBuf};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct FileSelectState {
    pub scenes_state: ListState,
    pub prefabs_state: ListState,
    pub assets_state: ListState,
}

impl Screen {
    pub fn new_file_select(project: &ProjectFiles) -> Self {
        let mut scenes_state = ListState::default();
        let mut prefabs_state = ListState::default();
        let mut assets_state = ListState::default();

        if !project.scenes.is_empty() {
            scenes_state.select(Some(0));
        } else if !project.prefabs.is_empty() {
            prefabs_state.select(Some(0));
        } else if !project.assets.is_empty() {
            assets_state.select(Some(0));
        }

        Screen::FileSelect(FileSelectState {
            scenes_state,
            prefabs_state,
            assets_state,
        })
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let Screen::FileSelect(FileSelectState {
        scenes_state,
        prefabs_state,
        assets_state,
    }) = &mut state.active_screen else {
        unreachable!()
    };

    let size = f.get_available_size();
    let footer_text: &str;

    if state.project.is_empty() {
        let paragraph = Paragraph::new("Empty project").alignment(Alignment::Center);
        f.render_widget(paragraph, size);
        footer_text = "ctrl+q: quit";
    } else {
        let ratios: Vec<(u32, u32)> = {
            let mut ratios = vec![(0, 0); 3];
            let mut non_empty = 0;
            if !state.project.scenes.is_empty() {
                non_empty += 1;
                ratios[0].0 = 1;
            }
            if !state.project.prefabs.is_empty() {
                non_empty += 1;
                ratios[1].0 = 1;
            }
            if !state.project.assets.is_empty() {
                non_empty += 1;
                ratios[2].0 = 1;
            }
            for mut ratio in ratios.iter_mut() {
                ratio.1 = non_empty;
            }
            ratios
        };
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(ratios[0].0, ratios[0].1),
                    Constraint::Ratio(ratios[1].0, ratios[1].1),
                    Constraint::Ratio(ratios[2].0, ratios[2].1),
                ]
                .as_ref(),
            )
            .split(size);

        let scenes_list =
            project_files_item_list(&state.project.scenes, &state.project.base_path, "Scenes");
        let prefabs_list =
            project_files_item_list(&state.project.prefabs, &state.project.base_path, "Prefabs");
        let assets_list =
            project_files_item_list(&state.project.assets, &state.project.base_path, "Assets");

        f.render_stateful_widget(scenes_list, layout[0], scenes_state);
        f.render_stateful_widget(prefabs_list, layout[1], prefabs_state);
        f.render_stateful_widget(assets_list, layout[2], assets_state);
        footer_text = "shift+j/k/down/up: switch section  j/k/down/up: move  space/enter: select  ctrl+q: quit";
    }

    f.render_footer(footer_text);
}

fn project_files_item_list<'a>(
    files: &'a [PathBuf],
    base_path: &'a Path,
    title: &'a str,
) -> List<'a> {
    let items: Vec<ListItem> = files
        .iter()
        .map(|item| {
            let path = fs::path_to_relative(item, base_path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            ListItem::new(path).style(Style::reset())
        })
        .collect();
    bordered_list(items, Some(title))
}

pub fn handle_event(event: &Event, state: &mut AppState) -> Result<(), Error> {
    let Screen::FileSelect(FileSelectState {
        scenes_state,
        prefabs_state,
        assets_state,
    }) = &mut state.active_screen else {
        unreachable!()
    };

    if let Event::Key(e) = event {
        match e {
            KeyEvent {
                code: KeyCode::Char('J') | KeyCode::Down,
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                if scenes_state.selected().is_some() {
                    if !state.project.prefabs.is_empty() {
                        scenes_state.select(None);
                        prefabs_state.select(Some(0));
                    } else if !state.project.assets.is_empty() {
                        scenes_state.select(None);
                        assets_state.select(Some(0));
                    }
                } else if prefabs_state.selected().is_some() {
                    if !state.project.assets.is_empty() {
                        prefabs_state.select(None);
                        assets_state.select(Some(0));
                    } else if !state.project.scenes.is_empty() {
                        prefabs_state.select(None);
                        scenes_state.select(Some(0));
                    }
                } else if assets_state.selected().is_some() {
                    if !state.project.scenes.is_empty() {
                        assets_state.select(None);
                        scenes_state.select(Some(0));
                    } else if !state.project.prefabs.is_empty() {
                        assets_state.select(None);
                        prefabs_state.select(Some(0));
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Char('K') | KeyCode::Up,
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                if scenes_state.selected().is_some() {
                    if !state.project.assets.is_empty() {
                        scenes_state.select(None);
                        assets_state.select(Some(0));
                    } else if !state.project.prefabs.is_empty() {
                        scenes_state.select(None);
                        prefabs_state.select(Some(0));
                    }
                } else if prefabs_state.selected().is_some() {
                    if !state.project.scenes.is_empty() {
                        prefabs_state.select(None);
                        scenes_state.select(Some(0));
                    } else if !state.project.assets.is_empty() {
                        prefabs_state.select(None);
                        assets_state.select(Some(0));
                    }
                } else if assets_state.selected().is_some() {
                    if !state.project.prefabs.is_empty() {
                        assets_state.select(None);
                        prefabs_state.select(Some(0));
                    } else if !state.project.scenes.is_empty() {
                        assets_state.select(None);
                        scenes_state.select(Some(0));
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Char('j') | KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                scenes_state.next_if_some(state.project.scenes.len());
                prefabs_state.next_if_some(state.project.prefabs.len());
                assets_state.next_if_some(state.project.assets.len());
            }
            KeyEvent {
                code: KeyCode::Char('k') | KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                scenes_state.prev_if_some(state.project.scenes.len());
                prefabs_state.prev_if_some(state.project.prefabs.len());
                assets_state.prev_if_some(state.project.assets.len());
            }
            KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                let selected_file_path = {
                    if let Some(idx) = scenes_state.selected() {
                        state.project.scenes[idx].clone()
                    } else if let Some(idx) = prefabs_state.selected() {
                        state.project.prefabs[idx].clone()
                    } else if let Some(idx) = assets_state.selected() {
                        state.project.assets[idx].clone()
                    } else {
                        PathBuf::new()
                    }
                };
                state.active_screen = Screen::new_file_view(selected_file_path)?;
            }
            _ => {}
        }
    }
    Ok(())
}
