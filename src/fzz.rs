use std::sync::mpsc::Receiver;

use crate::{
    AppArgs,
    error::Error,
    events::{Event, WorkerThreads},
    fuzzyfinder_widget::{FzzWidget, FzzWidgetState},
    tty::TTY,
};
use clap::Parser;
use crossterm::event;
use ratatui::widgets::StatefulWidget;

pub struct Fzz {
    exit: bool,
    events: Receiver<Event>,
    fzz_state: FzzWidgetState,
    tty: TTY,
}

impl Fzz {
    pub fn new() -> Self {
        let WorkerThreads { tx, rx } = WorkerThreads::init();
        let args = AppArgs::parse();

        Self {
            exit: false,
            events: rx,
            fzz_state: FzzWidgetState::new().set_args(&args).set_tx(tx),
            tty: TTY::new(),
        }
    }

    pub fn render(&mut self) -> Result<Option<String>, Error> {
        while !self.exit {
            self.draw()?;
            self.update()?;
        }

        self.tty.restore();

        Ok(self.fzz_state.get_selected())
    }

    pub fn update(&mut self) -> Result<(), Error> {
        match self.events.recv()? {
            Event::Input(key_event) => match key_event.kind {
                event::KeyEventKind::Press => match key_event.code {
                    event::KeyCode::Char(c) => {
                        self.fzz_state.push_char(c);
                    }
                    event::KeyCode::Backspace => {
                        self.fzz_state.pop_char();
                    }
                    event::KeyCode::Up => {
                        self.fzz_state.up();
                    }
                    event::KeyCode::Down => {
                        self.fzz_state.down();
                    }
                    event::KeyCode::Enter => {
                        self.exit = true;
                        self.fzz_state.select_item();
                    }
                    event::KeyCode::Esc => {
                        self.exit = true;
                    }
                    _ => (),
                },
                _ => (),
            },
            Event::AddList(s) => self.fzz_state.add_list(s),
            Event::RefreshList(v) => self.fzz_state.refresh_list(v),
        };

        Ok(())
    }

    pub fn draw(&mut self) -> Result<(), Error> {
        self.tty
            .terminal
            .draw(|frame| {
                FzzWidget::new().render(frame.area(), frame.buffer_mut(), &mut self.fzz_state);
            })
            .map_err(|e| Error::UnableToDraw { from: "main", e })?;

        Ok(())
    }
}
