mod error;
mod events;
mod fuzzyfinder_widget;
mod fzz;
mod tty;
mod utils;

use clap::Parser;
use events::Event;
use fzz::Fzz;

/// Simple Fuzzy finder for the terminal
#[derive(Parser)]
pub struct AppArgs {
    /// Delimiter to use to split the string
    #[arg(long, short)]
    pub delimiter: Option<char>,

    /// Case insensative search
    #[arg(long, short, action)]
    pub case_sensative: Option<bool>,

    /// Filter threshold 0.0 - 1.0
    #[arg(long, short)]
    pub threshold: Option<f64>,
}

fn main() {
    match Fzz::new().render() {
        Ok(Some(o)) => println!("{}", o),
        Err(e) => eprintln!("{}", e.to_string()),
        _ => (),
    };
}
