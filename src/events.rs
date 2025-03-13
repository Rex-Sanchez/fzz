use std::{
    io::{BufRead, BufReader},
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, KeyEvent};

use crate::Fzz;

pub enum Event {
    Input(KeyEvent),
    AddList(Vec<String>),
    RefreshList(Vec<(usize, String, f64)>),
}

impl Event {
    pub fn handle(self, app: &mut Fzz) {
        match self {
            Event::Input(key_event) => match key_event.kind {
                event::KeyEventKind::Press => match key_event.code {
                    event::KeyCode::Char(c) => {
                        app.fzz_state.push_char(c);
                    }
                    event::KeyCode::Backspace => {
                        app.fzz_state.pop_char();
                    }
                    event::KeyCode::Up => {
                        app.fzz_state.up();
                    }
                    event::KeyCode::Down => {
                        app.fzz_state.down();
                    }
                    event::KeyCode::Enter => {
                        app.exit = true;
                        app.fzz_state.select_item();
                    }
                    event::KeyCode::Esc => {
                        app.exit = true;
                    }

                    _ => (),
                },
                _ => (),
            },
            Event::AddList(s) => app.fzz_state.add_list(s),
            Event::RefreshList(v) => app.fzz_state.refresh_list(v),
        }
    }
}

impl Event {
    fn spaw_event_thread(tx: Sender<Event>) {
        loop {
            match event::read().expect("Failed to read event") {
                event::Event::Key(key_event) => match tx.send(Event::Input(key_event)) {
                    Ok(_) => (),
                    Err(e) => eprintln!("error sending message: {e}"),
                },
                _ => (),
            }
        }
    }
    fn spawn_stdin_thread(tx: Sender<Self>) {
        let stdin = BufReader::new(std::io::stdin());
        let mut lines = stdin.lines();
        let mut list: Vec<String> = Vec::new();

        let mut now = Instant::now();

        while let Some(Ok(line)) = lines.next() {
            list.push(line);

            if (now.elapsed() >= Duration::from_millis(100)) || (list.len() > 10000) {
                let _ = tx.send(Event::AddList(std::mem::take(&mut list)));
                now = Instant::now();
            }
        }

        if !list.is_empty() {
            let _ = tx.send(Event::AddList(std::mem::take(&mut list)));
        }
    }

    pub fn init() -> (Receiver<Self>, Sender<Self>) {
        let (event_tx, event_rx) = channel::<Event>();
        let key_event_tx = event_tx.clone();
        let std_event_tx = event_tx.clone();
        let _ = thread::spawn(|| Event::spaw_event_thread(key_event_tx));
        let _ = thread::spawn(|| Event::spawn_stdin_thread(std_event_tx));

        (event_rx, event_tx)
    }
}
