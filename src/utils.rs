use std::{ops::Deref, sync::mpsc::{channel, Receiver, Sender}, thread};




pub struct Job<D, T> {
    pub tx: Option<Sender<T>>,
    pub data: D,
}

impl<D, T > Job<D, T>
where
    D: 'static,
    T: 'static,
{
    pub fn new(data: D) -> Self {
        Self { data, tx: None }
    }
    pub fn tx(mut self, tx: Sender<T>) -> Self {
        self.tx = Some(tx);
        self
    }
    pub fn spawn(mut self, f: fn(Self)) -> Option<Receiver<T>> {
        if self.tx.is_some() {
            thread::spawn(move || f);
            return None;
        } else {
            let (tx, rx) = channel();
            self.tx = Some(tx);
            thread::spawn(move || f);
            return Some(rx);
        }
    }
    pub fn send(&self, message: T) {
        let _ = self.tx.as_ref().expect("This should not happen as tx should always be set").send(message);
    }
}

impl<D,T > Deref for Job<D, T> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}


