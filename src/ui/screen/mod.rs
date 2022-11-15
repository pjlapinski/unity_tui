pub mod file_select;
pub mod file_view;

use crate::ui::screen::file_select::FileSelectState;
use crate::ui::screen::file_view::FileViewState;
use tui::widgets::ListState;
use tui::{backend::Backend, layout::Rect, Frame};

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
