use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;

use ratatui::prelude::*;

use ratatui::widgets::Paragraph;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListState, StatefulWidget},
};

use rayon::iter::{
    IndexedParallelIterator,  IntoParallelRefIterator,
     ParallelIterator,
};
use rayon::slice::ParallelSliceMut;
use rayon::str::ParallelString;

use rust_fuzzy_search::fuzzy_compare;

use crate::AppArgs;
use crate::events::Event;

pub struct FzzWidget;

impl FzzWidget {
    pub fn new() -> Self {
        Self
    }
    fn draw(self, area: Rect, buf: &mut Buffer, state: &mut FzzWidgetState) {
        let [list_search_area, search_box_area] = Self::generate_layout(area);

        Self::draw_search_list(list_search_area, buf, state);
        Self::draw_search_box(search_box_area, buf, state);
    }
    fn generate_layout(area: Rect) -> [Rect; 2] {
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area)
    }
    fn draw_search_list(area: Rect, buf: &mut Buffer, state: &mut FzzWidgetState) {
        let list = state
            .sorted_list_items
            .par_iter()
            .map(|(_index, value, _score)| format!("{}", value))
            .collect::<Vec<String>>();

        StatefulWidget::render(
            List::new(list)
                .highlight_symbol("|> ")
                .highlight_style(
                    Style::new()
                        .bg(Color::Rgb(255, 175, 0))
                        .add_modifier(Modifier::BOLD)
                        .black(),
                )
                .block(
                    Block::bordered()
                        .title(format!(
                            "{}/{}",
                            state.sorted_list_items.len(),
                            state.stdin.read().expect("this should not fail").split(&state.delimiter.clone().to_string()).count()
                        ))
                        .title_alignment(ratatui::layout::Alignment::Right),
                )
                .direction(ratatui::widgets::ListDirection::BottomToTop),
            area,
            buf,
            &mut state.state,
        );
    }
    fn draw_search_box(area: Rect, buf: &mut Buffer, state: &mut FzzWidgetState) {
        Paragraph::new(state.search_input.clone())
            .block(Block::bordered().title("Search"))
            .render(area, buf);
    }
}

impl StatefulWidget for FzzWidget {
    type State = FzzWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.draw(area, buf, state);
    }
}

pub struct FzzWidgetState {
    tx: Option<Sender<Event>>,
    selected_index: Option<usize>,
    search_input: String,
    stdin: Arc<RwLock<String>>,
    sorted_list_items: Vec<(usize, String, f64)>,
    state: ListState,
    delimiter: char,
    case_insesative: bool,
    threshold: f64,
}

impl Default for FzzWidgetState {
    fn default() -> Self {
        Self {
            tx: Default::default(),
            selected_index: Default::default(),
            search_input: Default::default(),
            stdin: Default::default(),
            sorted_list_items: Default::default(),
            state: Default::default(),
            delimiter: '\n',
            case_insesative: true,
            threshold: 0.1,
        }
    }
}

impl FzzWidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    // This should be set to allow the list to be updated from its thread 
    pub fn set_tx(&mut self, sender: Sender<Event>) {
        self.tx = Some(sender);
    }



    // ------------ | List control logic | -----------
    
    // add char to search input 
    pub fn push_char(&mut self, c: char) {
        self.search_input.push_str(c.to_string().as_str());
        self.should_refresh();
    }

    // remove a char from the input
    pub fn pop_char(&mut self) {
        self.search_input.pop();
        self.should_refresh();
    }

    // move the cursor up to the next item
    pub fn up(&mut self) {
        self.state.select_next();
    }

    // move the cursor down to the prev item
    pub fn down(&mut self) {
        self.state.select_previous();
    }
    
    // ------------ | Refresh logic | -----------


    // when the shoud_refresh sorting algo is done and sends the sorted list via channel this
    // function is called to set the actual list in the state
    pub fn refresh_list(&mut self, v: Vec<(usize, String, f64)>) {
        self.sorted_list_items = v;
        self.state.select_first();
    }


    // this function is called when the list needs to refreshed as in when there hes been search
    // input change, or when a chuck is received from stdin when the sort is dont the list is send
    // over channel with event Event::RefreshList 
    pub fn should_refresh(&self) {
        let list = self.stdin.clone();
        let case_sensative = self.case_insesative.clone();
        let search_text = self.search_input.clone();
        let threshold = self.threshold.clone();
        let delim = self.delimiter.clone();
        

        let tx = self
            .tx
            .clone()
            .expect("This should not happen if tx is set for the fuzzyfinder");

        thread::spawn(move || {
            let mut sorted_list = list.read().expect("this should not fail")
                .par_split(delim)
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .par_iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    let mut value = value.clone();

                    if !case_sensative {
                        value = value.to_lowercase();
                    }

                    let score = if !search_text.is_empty() {
                        fuzzy_compare(&search_text, &value) as f64
                    } else {
                        0.0
                    };

                    if search_text.len() < 3  || (score > threshold) {
                        Some((index, value.clone(), score))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            sorted_list.par_sort_by(|a, b| {
                let cmp = b.2.partial_cmp(&a.2).unwrap();
                if cmp == std::cmp::Ordering::Equal {
                    a.1.len().cmp(&b.1.len())
                } else {
                    cmp
                }
            });
            let _ = tx.send(crate::Event::RefreshList(sorted_list));
        });
    }



    // ------------ | Selectin logic | -----------

    // call this function to select a item from the list, this can then be retreived via
    // self.get_selected
    pub fn select_item(&mut self) {
        if let Some(index) = self.state.selected() {
            self.selected_index = self.sorted_list_items.get(index).map(|i| i.0)
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        self.stdin.read().expect("this should not fail")
            .split(&self.delimiter.to_string())
            .collect::<Vec<_>>()
            .get(self.selected_index?)
            .map(|f| f.to_string())
    }



    // ------------ | Setup logic | -----------

    pub fn set_args(mut self, args: &mut AppArgs) -> Self {
        self.threshold = args.threshold.unwrap_or(0.2);
        self.case_insesative = args.case_sesative.unwrap_or(false);
        self.delimiter = args.delimiter.unwrap_or('\n');
        self
    }

    pub fn add_list(&mut self, s: Vec<String>) {
        {
            self.stdin.write().expect("this should not fail").push_str(&s.join("\n"));
        }
        self.should_refresh();
    }
}
