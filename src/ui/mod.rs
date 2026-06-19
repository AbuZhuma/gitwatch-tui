mod dashboard;
mod widgets;

use ratatui::Frame;

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame) {
    dashboard::render(app, frame);
}
