use fuzzy_search::{automata::LevenshteinAutomata, symspell::SymSpell};
use log::info;

use crate::{
    ansi::{AnsiCommand, AnsiParser},
    measure::TextMeasureCache,
    FONT_SIZE,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub commands: Vec<AnsiCommand>,
    pub unbroken_line_index: usize,
}

impl ConsoleLine {
    pub fn new(commands: Vec<AnsiCommand>, unbroken_line_index: usize) -> Self {
        Self {
            commands,
            unbroken_line_index,
        }
    }
}

#[derive(Clone)]
pub struct LineManager {
    unbroken_lines: Vec<Vec<AnsiCommand>>,
    measure_cache: Rc<RefCell<TextMeasureCache>>,
    canvas_width: f64,
    cached_lines: Option<Vec<ConsoleLine>>,
    search_results: Option<Vec<usize>>,
}

impl LineManager {
    pub fn new(
        measure_cache: &Rc<RefCell<TextMeasureCache>>,
        canvas_width: f64,
    ) -> Self {
        Self {
            unbroken_lines: Vec::new(),
            cached_lines: None,
            measure_cache: measure_cache.clone(),
            canvas_width,
            search_results: None,
        }
    }

    pub fn add_line(&mut self, line: &str) -> usize {
        if self.unbroken_lines.len() > 1000 {
            self.remove_at(0);
        }

        let parser = AnsiParser::new(line.to_owned());
        let result = parser.parse();
        self.unbroken_lines.push(result.clone());
        if let Some(cache) = &mut self.cached_lines {
            let index = self.unbroken_lines.len() - 1;
            let new_broken_lines = Self::calculate_line_breaks(
                result.clone(),
                &self.measure_cache,
                self.canvas_width,
            );
            for broken_line in new_broken_lines {
                cache.push(ConsoleLine::new(broken_line, index));
            }
        } else {
            self.invalidate_cache();
        }

        Self::calculate_line_breaks(
            result.clone(),
            &self.measure_cache,
            self.canvas_width,
        )
        .len()
    }

    pub fn invalidate_cache(&mut self) {
        self.cached_lines = None;
    }

    pub fn calculate_line_breaks(
        line: Vec<AnsiCommand>,
        measure_cache: &Rc<RefCell<TextMeasureCache>>,
        canvas_width: f64,
    ) -> Vec<Vec<AnsiCommand>> {
        let mut lines: Vec<Vec<AnsiCommand>> = Vec::new();
        let mut x = 0.0;
        let mut current_line = Vec::new();
        let mut last_was_space = false;
        for command in line.iter() {
            match command {
                AnsiCommand::RenderText(text) => {
                    let split = text.split(' ').collect::<Vec<&str>>();
                    for word in split.iter() {
                        let word = if word.is_empty() && !last_was_space {
                            last_was_space = true;
                            " "
                        } else {
                            last_was_space = false;
                            *word
                        };
                        let word = (*word).to_owned();
                        let word = word.as_str();
                        let width =
                            measure_cache.borrow_mut().measure(FONT_SIZE, word);
                        if x + width > canvas_width {
                            lines.push(current_line.clone());
                            let mut styles = Vec::new();
                            for style in current_line.iter().rev() {
                                match style {
                                    AnsiCommand::RenderText(_) => continue,
                                    _ => styles.push(style.clone()),
                                }
                            }
                            current_line.clear();
                            for style in styles.iter().rev() {
                                current_line.push(style.clone());
                            }
                            x = 0.0;
                        }
                        current_line
                            .push(AnsiCommand::RenderText(word.to_owned()));
                        x += width;
                    }
                }
                _ => {
                    current_line.push(command.clone());
                }
            }
        }

        if current_line.len() > 0 {
            lines.push(current_line);
        }

        lines
    }

    pub fn len_no_linebreaks(&self) -> usize {
        self.unbroken_lines.len()
    }

    pub fn len_linebreaks(&mut self) -> usize {
        if self.cached_lines.is_none() {
            self.recalculate_cache();
        }
        if self.search_results.is_some() {
            return self.search_results.as_ref().unwrap().len();
        }
        self.cached_lines.as_ref().unwrap().len()
    }

    pub fn get_lines(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> Vec<ConsoleLine> {
        if self.cached_lines.is_none() {
            self.recalculate_cache();
        }
        if self.search_results.is_some() {
            let search_results = self.search_results.as_ref().unwrap();
            return search_results
                .iter()
                .filter_map(|index| {
                    if range.contains(index) {
                        Some(
                            self.cached_lines.as_ref().unwrap()[*index].clone(),
                        )
                    } else {
                        None
                    }
                })
                .collect();
        }
        self.cached_lines.as_ref().unwrap()[range].to_vec()
    }

    fn recalculate_cache(&mut self) {
        self.cached_lines = Some(
            self.unbroken_lines
                .iter()
                .enumerate()
                .flat_map(|(index, line)| {
                    Self::calculate_line_breaks(
                        line.clone(),
                        &self.measure_cache,
                        self.canvas_width,
                    )
                    .into_iter()
                    .map(move |broken_line| {
                        ConsoleLine::new(broken_line, index)
                    })
                })
                .collect(),
        );
    }

    pub fn clear(&mut self) {
        self.unbroken_lines.clear();
        self.cached_lines = None;
    }

    pub fn on_resize(&mut self, width: f64) {
        self.canvas_width = width;
        self.invalidate_cache();
    }

    pub fn remove_at(&mut self, raw_index: usize) {}

    pub fn search(&mut self, query: &str) {
        let len = self.len_linebreaks();
        let all_lines = self.get_lines(0..len);

        let choices = all_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                (
                    i,
                    line.commands
                        .iter()
                        .filter_map(|command| match command {
                            AnsiCommand::RenderText(text) => Some(text),
                            _ => None,
                        })
                        .map(String::as_str)
                        .collect::<Vec<&str>>()
                        .join(""),
                )
            })
            .filter(|(_, line)| {
                line.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        self.search_results = Some(choices);

        // scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        // let top_indices = scores
        //     .iter()
        //     .filter(|(_, score)| *score > 0.0)
        //     .take(20)
        //     .map(|(index, _)| *index)
        //     .collect::<Vec<_>>();

        // self.search_results = Some(top_indices.clone());
    }

    pub fn clear_search(&mut self) {
        self.search_results = None;
    }
}
