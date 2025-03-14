mod events;
mod fuzzyfinder_widget;
mod tty;
mod utils;

use events::Event;
use fuzzyfinder_widget::{FzzWidget, FzzWidgetState};
use tty::TTY;

use clap::Parser;
use ratatui::{Frame, widgets::StatefulWidget};



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

pub struct Fzz {
    exit: bool,
    fzz_state: FzzWidgetState,
}

impl Fzz {
    fn new(args: &mut AppArgs) -> Self {
        Self {
            exit: false,
            fzz_state: FzzWidgetState::new().set_args(args),
        }
    }

    fn run(&mut self) -> Option<String> {
        let mut tty = TTY::new();
        let (events, sender) = Event::init();
    
        self.fzz_state.set_tx(sender);
    
        while !self.exit {
            tty.terminal
                .draw(|frame| self.draw(frame))
                .expect("Faild to draw");

            if let Ok(event) = events.recv() {
                event.handle(self);
            }
        }

        tty.restore();

        self.fzz_state.get_selected()
    }
    fn draw(&mut self, frame: &mut Frame) {
        FzzWidget::new().render(frame.area(), frame.buffer_mut(), &mut self.fzz_state);
    }
}
