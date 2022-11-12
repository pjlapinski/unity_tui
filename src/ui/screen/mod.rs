pub mod file_select;

use crate::ui::screen::file_select::FileSelectState;
use tui::widgets::ListState;
use tui::{backend::Backend, layout::Rect, Frame};

pub enum Screen {
    FileSelect(FileSelectState),
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

pub(self) fn get_available_size<B: Backend>(f: &Frame<B>) -> Rect {
    let s = f.size();
    Rect {
        x: s.x,
        y: s.y,
        width: s.width,
        height: s.height - 1,
    }
}

pub(self) fn render_footer<B: Backend>(f: &mut Frame<B>, text: &str) {
    use tui::{layout::Alignment, widgets::Paragraph};

    let size = f.size();

    let footer = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(
        footer,
        Rect {
            x: 0,
            y: size.height - 1,
            width: size.width,
            height: 1,
        },
    );
}
