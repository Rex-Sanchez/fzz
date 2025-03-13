use std::fs::File;

use crossterm::{cursor, execute, terminal::{self, Clear}};
use ratatui::{prelude::CrosstermBackend, Terminal};

pub struct TTY {
    tty: File,
    pub terminal: Terminal<CrosstermBackend<File>>,
}
impl TTY {
    pub fn new() -> Self {
        let tty = Self::init();

        let  backend = CrosstermBackend::new(tty.try_clone().unwrap());
        let  terminal = Terminal::new(backend).unwrap();

        Self { tty, terminal }
    }
    fn init() -> File {
        let _ = terminal::enable_raw_mode();

        let mut tty = std::fs::File::options()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .unwrap();

        execute!(tty, terminal::EnterAlternateScreen).unwrap();

        return tty;
    }
    pub fn restore(&mut self) {
        let _ = terminal::disable_raw_mode();

        execute!(
            self.tty,
            terminal::LeaveAlternateScreen,
            Clear(terminal::ClearType::All)
        )
        .unwrap();

        execute!(
            self.tty,
            Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();
    }
}
