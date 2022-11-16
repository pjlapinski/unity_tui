pub mod file_select;
pub mod file_view;

use crate::ui::screen::{file_select::FileSelectState, file_view::FileViewState};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub enum Screen {
    FileSelect(FileSelectState),
    FileView(FileViewState),
}

pub trait SelectNextPrev {
    fn next_if_some(&mut self, max: usize);
    fn prev_if_some(&mut self, max: usize);
}

impl SelectNextPrev for ListState {
    fn next_if_some(&mut self, max: usize) {
        if let Some(idx) = self.selected() {
            self.select(Some((idx + 1) % max));
        }
    }

    fn prev_if_some(&mut self, max: usize) {
        if let Some(mut idx) = self.selected() {
            if idx == 0 {
                idx = max;
            }
            idx -= 1;
            self.select(Some(idx));
        }
    }
}

pub(self) fn bordered_list<'a, T, U>(items: T, title: Option<U>) -> List<'a>
where
    T: Into<Vec<ListItem<'a>>>,
    U: Into<Spans<'a>>,
{
    let mut block = Block::default().borders(Borders::ALL);
    if let Some(t) = title {
        block = block.title(t);
    }
    List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::White)
            .fg(Color::Black)
            .add_modifier(Modifier::ITALIC),
    )
}

pub(self) trait AvailableSize<B: Backend> {
    fn get_available_size(&self) -> Rect;
}

impl<'a, B: Backend> AvailableSize<B> for Frame<'a, B> {
    fn get_available_size(&self) -> Rect {
        let s = self.size();
        Rect {
            x: s.x,
            y: s.y,
            width: s.width,
            height: s.height - 1,
        }
    }
}

pub(self) trait FooterRenderer<B: Backend> {
    fn render_footer(&mut self, text: &str);
}

impl<'a, B: Backend> FooterRenderer<B> for Frame<'a, B> {
    fn render_footer(&mut self, text: &str) {
        use tui::{layout::Alignment, widgets::Paragraph};

        let size = self.size();

        let footer = Paragraph::new(text).alignment(Alignment::Center);
        self.render_widget(
            footer,
            Rect {
                x: 0,
                y: size.height - 1,
                width: size.width,
                height: 1,
            },
        );
    }
}
