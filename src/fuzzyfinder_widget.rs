use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::{iter, string};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use rayon::str::ParallelString;

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListState, StatefulWidget},
};

use crate::AppArgs;
use crate::events::Event;
use crate::utils::{Job, contains_fuzzy_search, trigram_fuzzy_compare};

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
        StatefulWidget::render(
            List::new(state.get_sorted_list())
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
                            state.sorted_list.len(),
                            state
                                .stdin
                                .read()
                                .expect("this should not fail")
                                .par_split(state.delimiter)
                                .count()
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

pub type SortedList = Vec<SortedListItem>;

pub struct SortedListItem {
    text: String,
    index: usize,
    score: f64,
}

pub struct FzzWidgetState {
    tx: Option<Sender<Event>>,
    selected_index: Option<usize>,
    search_input: String,
    stdin: Arc<RwLock<String>>,
    sorted_list: SortedList,
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
            sorted_list: Default::default(),
            state: Default::default(),
            delimiter: Default::default(),
            case_insesative: Default::default(),
            threshold: Default::default(),
        }
    }
}

impl FzzWidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    /// This should be set to allow the list to be updated from its thread
    pub fn set_tx(mut self, sender: Sender<Event>) -> Self {
        self.tx = Some(sender);
        self
    }

    // ------------ | List control logic | -----------

    /// add char to search input
    pub fn push_char(&mut self, c: char) {
        self.search_input.push_str(c.to_string().as_str());
        self.should_refresh();
    }

    /// remove a char from the input
    pub fn pop_char(&mut self) {
        self.search_input.pop();
        self.should_refresh();
    }

    /// move the cursor up to the next item
    pub fn up(&mut self) {
        self.state.select_next();
    }

    /// move the cursor down to the prev item
    pub fn down(&mut self) {
        self.state.select_previous();
    }

    // ------------ | Refresh logic | -----------

    // when the shoud_refresh sorting algo is done and sends the sorted list via channel this
    // function is called to set the actual list in the state
    pub fn refresh_list(&mut self, v: SortedList) {
        self.sorted_list = v;
        self.state.select_first();
    }

    // this function is called when the list needs to refreshed as in when there hes been search
    // input change, or when a chuck is received from stdin when the sort is dont the list is send
    // over channel with event Event::RefreshList
    pub fn should_refresh(&self) {
        #[derive(Debug)]
        struct Data {
            list: Arc<RwLock<String>>,
            case_sensative: bool,
            search_text: String,
            threshold: f64,
            delim: char,
        }

        Job::new(Data {
            list: self.stdin.clone(),
            case_sensative: self.case_insesative,
            search_text: self.search_input.clone(),
            threshold: self.threshold,
            delim: self.delimiter.clone(),
        })
        .tx(self
            .tx
            .clone()
            .expect("This should not happen if tx is set for the fuzzyfinder"))
        .spawn(|s| {
            let mut sorted_list = s
                .list
                .read()
                .expect("this should not fail")
                .par_split(s.delim)
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .par_iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    let mut value = value.clone();

                    if !s.case_sensative {
                        value = value.to_lowercase();
                    }

                    let score = if !s.search_text.is_empty() {
                        if s.search_text.len() < 3 {
                            contains_fuzzy_search(&s.search_text, &value) as f64
                        } else {
                            trigram_fuzzy_compare(&s.search_text, &value) as f64
                        }
                    } else {
                        0.0
                    };

                    if s.search_text.is_empty() || (score > s.threshold) {
                        Some(SortedListItem {
                            text: value,
                            index,
                            score,
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            sorted_list.par_sort_by(|a, b| {
                // Sort be score
                let cmp = b.score.partial_cmp(&a.score).unwrap();

                // if scores are equal it should short by string len "shortist on top"
                if cmp == std::cmp::Ordering::Equal {
                    a.text.len().cmp(&b.text.len())
                } else {
                    cmp
                }
            });
            let _ = s.send(crate::Event::RefreshList(sorted_list));
        });
    }

    // ------------ | Selectin logic | -----------

    // call this function to select a item from the list,
    // this can then be retreived via
    // self.get_selected
    pub fn select_item(&mut self) {
        if let Some(index) = self.state.selected() {
            self.selected_index = self.sorted_list.get(index).map(|i| i.index)
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        self.stdin
            .read()
            .expect("this should not fail")
            .split(&self.delimiter.to_string())
            .collect::<Vec<_>>()
            .get(self.selected_index?)
            .map(|f| f.to_string())
    }

    fn get_sorted_list(&self) -> Vec<String> {
        self.sorted_list
            .par_iter()
            .map(|v| format!("{}", v.text))
            .collect::<Vec<String>>()
    }

    // ------------ | Setup logic | -----------

    pub fn set_args(mut self, args: &AppArgs) -> Self {
        self.threshold = args.threshold.unwrap_or(0.2);
        self.case_insesative = args.case_sesative.unwrap_or(false);
        self.delimiter = args.delimiter.unwrap_or('\n');
        self
    }

    pub fn add_list(&mut self, s: Vec<String>) {
        {
            self.stdin
                .write()
                .expect("this should not fail")
                .push_str(&s.join("\n"));
        }
        self.should_refresh();
    }
}
