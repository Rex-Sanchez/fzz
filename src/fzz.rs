use ratatui::{Frame, widgets::StatefulWidget};
use crate::{events::Event, fuzzyfinder_widget::{FzzWidget, FzzWidgetState}, tty::TTY, AppArgs};


pub struct Fzz {
    pub exit: bool,
    pub fzz_state: FzzWidgetState,
}

impl Fzz {
    pub fn new(args: &mut AppArgs) -> Self {
        Self {
            exit: false,
            fzz_state: FzzWidgetState::new().set_args(args),
        }
    }

    pub fn run(&mut self) -> Option<String> {
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
    pub fn draw(&mut self, frame: &mut Frame) {
        FzzWidget::new().render(frame.area(), frame.buffer_mut(), &mut self.fzz_state);
    }
}
