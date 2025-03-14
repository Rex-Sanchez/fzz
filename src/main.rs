mod events;
mod fuzzyfinder_widget;
mod tty;
mod utils;
mod fzz;

use events::Event;
use clap::Parser;
use fzz::Fzz;



/// Simple Fuzzy finder for the terminal
#[derive(Parser)]
pub struct AppArgs {
    
    /// Delimiter to use to split the string
    #[arg(long, short)]
    pub delimiter: Option<char>,
    
    /// Case insensative search
    #[arg(long, short, action)]
    pub case_sesative: Option<bool>,
    
    /// Filter threshold 0.0 - 1.0
    #[arg(long, short)]
    pub threshold: Option<f64>,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = AppArgs::parse();

    let mut fzz= Fzz::new(&mut args);

    if let Some(o) = fzz.run() {
        println!("{}", o);
    }

    Ok(())
}


