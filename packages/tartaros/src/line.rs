use log::info;
use rust_fuzzy_search::fuzzy_compare;

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

#[derive(Debug, Clone)]
pub struct LineManager {
    unbroken_lines: Vec<Vec<AnsiCommand>>,
    lines: Vec<ConsoleLine>,
    measure_cache: Rc<RefCell<TextMeasureCache>>,
    canvas_width: f64,
}

impl LineManager {
    pub fn new(
        measure_cache: &Rc<RefCell<TextMeasureCache>>,
        canvas_width: f64,
    ) -> Self {
        Self {
            lines: Vec::new(),
            unbroken_lines: Vec::new(),
            measure_cache: measure_cache.clone(),
            canvas_width,
        }
    }

    pub fn add_line(&mut self, line: &str) -> usize {
        if self.unbroken_lines.len() > 1000 {
            self.remove_at(0);
        }
        let parser = AnsiParser::new(line.to_owned());
        let result = parser.parse();
        self.unbroken_lines.push(result.clone());
        let new_lines = self.calculate_line_breaks(result);
        let index = self.unbroken_lines.len().saturating_sub(1);
        for line in new_lines.clone() {
            self.lines.push(ConsoleLine::new(line.clone(), index));
        }
        new_lines.len()
    }

    pub fn calculate_line_breaks(
        &self,
        line: Vec<AnsiCommand>,
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
                        let width = self
                            .measure_cache
                            .borrow_mut()
                            .measure(FONT_SIZE, word);
                        if x + width > self.canvas_width {
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

    pub fn len_linebreaks(&self) -> usize {
        self.lines.len()
    }

    pub fn get_lines(&self, range: std::ops::Range<usize>) -> Vec<ConsoleLine> {
        self.lines[range].to_vec()
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.unbroken_lines.clear();
    }

    pub fn on_resize(&mut self, width: f64) {
        self.canvas_width = width;
        self.lines.clear();
        for line in self.unbroken_lines.clone().iter() {
            let new_lines = self.calculate_line_breaks(line.clone());
            let index = self.lines.len().saturating_sub(1);
            for line in new_lines {
                self.lines.push(ConsoleLine::new(line, index));
            }
        }
    }

    pub fn remove_at(&mut self, raw_index: usize) {
        let mut index = 0;
        for line in self.lines.clone().iter() {
            if line.unbroken_line_index == raw_index {
                self.lines.remove(index);
            } else {
                if let Some(line) = self.lines.get_mut(index) {
                    line.unbroken_line_index -= 1;
                }
            }
            index += 1;
        }

        self.unbroken_lines.remove(raw_index);
    }

    pub fn search(&mut self, query: &str) {
        let mut scores = Vec::new();
        for (index, line) in self.unbroken_lines.iter().enumerate() {
            let text = line
                .iter()
                .filter_map(|command| match command {
                    AnsiCommand::RenderText(text) => Some(text),
                    _ => None,
                })
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .join("");

            info!("Searching in line: {}", text);

            let score =
                fuzzy_compare(&text.to_lowercase(), &query.to_lowercase());
            scores.push((index, score));
        }

        // info!("Scores: {:?}", scores);
        // sort score by score, return top 20
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_scores = scores
            .iter()
            .filter(|x| x.1 != 0.0)
            .take(20)
            .collect::<Vec<_>>();
    }
}
