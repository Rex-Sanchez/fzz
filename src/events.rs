use std::{
    io::{BufRead, BufReader},
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, KeyEvent},
    tty::IsTty,
};

use crate::fuzzyfinder_widget::SortedList;

pub enum Event {
    Input(KeyEvent),
    AddList(Vec<String>),
    RefreshList(SortedList),
    NoStdin,
}

pub struct WorkerThreads {
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}

impl WorkerThreads {
    pub fn init() -> Self {
        let (event_tx, event_rx) = channel::<Event>();
        let key_event_tx = event_tx.clone();
        let std_event_tx = event_tx.clone();
        let _ = thread::spawn(|| Self::spawn_event_thread(key_event_tx));
        let _ = thread::spawn(|| Self::spawn_stdin_thread(std_event_tx));

        Self {
            tx: event_tx,
            rx: event_rx,
        }
    }
    fn spawn_event_thread(tx: Sender<Event>) {
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
    fn spawn_stdin_thread(tx: Sender<Event>) {
        let stdin = std::io::stdin();

        if !stdin.is_tty() {
            let stdin = BufReader::new(stdin);
            let mut lines = stdin.lines();
            let mut list: Vec<String> = Vec::new();

            let mut now = Instant::now();

            while let Some(Ok(line)) = lines.next() {
                list.push(line);

                if now.elapsed() >= Duration::from_millis(100) {
                    let _ = tx.send(Event::AddList(std::mem::take(&mut list)));
                    now = Instant::now();
                }
            }

            if !list.is_empty() {
                let _ = tx.send(Event::AddList(std::mem::take(&mut list)));
            }
        }
        else{
            let _ = tx.send(Event::NoStdin);

        }
    }
}
