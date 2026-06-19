#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub viewer: String,
}

impl App {
    pub fn new(viewer: String) -> Self {
        Self {
            running: true,
            viewer,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
