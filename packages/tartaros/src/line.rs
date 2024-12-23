use std::{cell::RefCell, rc::Rc};

use log::info;

use crate::{
    ansi::{AnsiCommand, AnsiParser},
    measure::TextMeasureCache,
    FONT_SIZE, LINES_VISIBLE,
};

#[derive(Debug, Clone)]
pub struct LineManager {
    unbroken_lines: Vec<Vec<AnsiCommand>>,
    lines: Vec<Vec<AnsiCommand>>,
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
        let parser = AnsiParser::new(line.to_owned());
        let result = parser.parse();
        self.unbroken_lines.push(result.clone());
        let new_lines = self.calculate_line_breaks(result);
        for line in new_lines.clone() {
            // let len = self.len();
            self.lines.push(line);
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

    pub fn get_lines(
        &self,
        range: std::ops::Range<usize>,
    ) -> Vec<Vec<AnsiCommand>> {
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
            for line in new_lines {
                self.lines.push(line);
            }
        }
    }
}
