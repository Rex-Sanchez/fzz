use std::{
    iter,
    ops::Deref,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
};


#[derive(Debug)]
pub struct Job<D, T> {
    pub tx: Option<Sender<T>>,
    pub data: D,
}

impl<D, T> Job<D, T>
where
    D: 'static + Send,
    T: 'static + Send,
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
            thread::spawn(move || f(self));
            return None;
        } else {
            let (tx, rx) = channel();
            self.tx = Some(tx);
            thread::spawn(move || f(self));
            return Some(rx);
        }
    }
    pub fn send(&self, message: T) {
        let _ = self
            .tx
            .as_ref()
            .expect("This should not happen as tx should always be set")
            .send(message);
    }
}

impl<D, T> Deref for Job<D, T> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

fn trigrams(s: &str) -> Vec<Vec<char>> {
    let it_1 = iter::once(' ').chain(iter::once(' ')).chain(s.chars());
    let it_2 = iter::once(' ').chain(s.chars());
    let it_3 = s.chars().chain(iter::once(' '));

    let res: Vec<Vec<char>> = it_1
        .zip(it_2)
        .zip(it_3)
        .map(|((a, b), c)| vec![a, b, c])
        .collect();
    res
}

pub fn trigram_fuzzy_compare(a: &str, b: &str) -> f32 {
    let string_len = a.chars().count() + 1;

    let trigrams_a = trigrams(a);
    let trigrams_b = trigrams(b);

    let mut acc: f32 = 0.0f32;
    for t_a in &trigrams_a {
        for t_b in &trigrams_b {
            if t_a == t_b {
                acc += 1.0f32;
                break;
            }
        }
    }
    let res = acc / (string_len as f32);

    if (0.0f32..=1.0f32).contains(&res) {
        res
    } else {
        0.0f32
    }
}
pub fn contains_fuzzy_search(a: &str, b: &str) -> f32 {
    let s_a = a.split("").collect::<Vec<&str>>();
    let s_b = b.split("").collect::<Vec<&str>>();

    let a_len = a.len();

    let mut acc = 0;

    for i in s_a.iter() {
        for j in s_b.iter() {
            if i == j {
                acc += 1;
            }
        }
    }

    let res = a_len as f32 / acc as f32 ;
    if (0.0f32..=1.0f32).contains(&res) {
        1f32 - res
    } else {
        0.0f32
    }
}
