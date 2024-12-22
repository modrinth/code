/**
IMPORTANT INFO FOR FUTURE CONTRIBUTORS

Each call to any function on `ctx` is a call to a JS function. This is very slow.
You need to minimize this -- see `measure.rs` for details.
*/
mod ansi;
mod measure;
mod utils;

use std::{cell::RefCell, cmp, rc::Rc};

use ansi::{AnsiCommand, AnsiParser};
use log::Level;
use measure::TextMeasureCache;
use utils::{
    cancel_animation_frame, request_animation_frame, set_panic_hook, worker,
};
use wasm_bindgen::prelude::*;
use web_sys::{
    DedicatedWorkerGlobalScope, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};

const FONT_SIZE: usize = 18;
const LINE_HEIGHT: usize = FONT_SIZE + (FONT_SIZE / 2);
const LINES_VISIBLE: usize = 18;
const BOTTOM_MARGIN: usize = 20;
const CANVAS_HEIGHT: usize = (LINE_HEIGHT * (LINES_VISIBLE)) + BOTTOM_MARGIN;
const LINE_OFFSET: isize = 0;
const GAP_LINES: usize = 2;
const SCROLL_BAR_WIDTH: usize = 8;
const MIN_SCROLL_BAR_HEIGHT: usize = 16;

#[derive(Clone, Debug)]
struct PyroConsoleLine {
    ansi_commands: Vec<AnsiCommand>,
    raw_line: Vec<AnsiCommand>,
}

impl PyroConsoleLine {
    fn contains(&self, query: &str) -> bool {
        // for command in self.ansi_commands.iter() {
        //     match command {
        //         AnsiCommand::RenderText(text) => {
        //             if text.contains(query) {
        //                 return true;
        //             }
        //         }
        //         _ => continue,
        //     }
        // }
        // false

        let mut text = String::new();
        for command in self.ansi_commands.iter() {
            match command {
                AnsiCommand::RenderText(t) => text.push_str(t),
                _ => continue,
            }
        }
        text.contains(query)
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PyroConsoleState {
    animation_frame: i32,
    canvas: OffscreenCanvas,
    ctx: OffscreenCanvasRenderingContext2d,
    current_fill_style: String,
    lines: Vec<PyroConsoleLine>,
    last_size: (u32, u32),
    measure_cache: TextMeasureCache,
    query: String,
    offset: u64,
    last_frame_time: f64,
    framerates: Vec<f64>,
    fps: u16,
    scroll_bar_y_offset: f64,
    worker_scope: DedicatedWorkerGlobalScope,
}

#[wasm_bindgen]
pub struct PyroConsole {
    state: Rc<RefCell<PyroConsoleState>>,
}

#[wasm_bindgen]
impl PyroConsole {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: OffscreenCanvas) -> PyroConsole {
        set_panic_hook();
        _ = console_log::init_with_level(Level::Debug);
        let ctx = canvas
            .get_context("2d")
            .expect("Failed to get canvas ctx")
            .expect("Failed to get canvas ctx")
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .unwrap();
        PyroConsole {
            state: Rc::new(RefCell::new(PyroConsoleState {
                animation_frame: 0,
                canvas: canvas.clone(),
                ctx: ctx.clone(),
                current_fill_style: "black".to_owned(),
                lines: Vec::new(),
                last_size: (0, 0),
                offset: 0,
                query: String::new(),
                last_frame_time: 0.0,
                framerates: Vec::new(),
                fps: 360,
                scroll_bar_y_offset: -1.0,
                worker_scope: worker(),
                measure_cache: TextMeasureCache::new(ctx),
            })),
            // window: Rc::new(window()),
        }
    }

    pub fn init(&mut self) {
        let mut _state = &self.state;

        let closure = {
            let state = _state.clone();
            Closure::<dyn FnMut()>::new(move || {
                let mut state = state.borrow_mut();
                let new_fps = state.framerates.iter().sum::<f64>()
                    / state.framerates.len() as f64;
                state.fps = cmp::min(new_fps.round() as u16, 360) as u16;
                state.framerates.clear();
            })
        };

        _state
            .borrow()
            .worker_scope
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            )
            .expect("failed to set interval for fps");

        closure.forget();

        self.state.borrow().canvas.set_height(CANVAS_HEIGHT as u32);

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let closure = {
            let state = self.state.clone();
            Closure::<dyn FnMut()>::new(move || {
                let mut state = state.borrow_mut();
                if (state.offset as usize) >= state.lines.len() {
                    state.offset = 0;
                }
                let performance = state
                    .worker_scope
                    .performance()
                    .expect("failed to get performance");
                PyroConsole::check_for_resize(&mut state);
                PyroConsole::draw_console(&mut state);
                let now = performance.now();
                let delta = now - state.last_frame_time;
                state.last_frame_time = now;
                state.framerates.push(1000.0 / delta);
                state.animation_frame =
                    request_animation_frame(f.borrow().as_ref().unwrap());
            })
        };

        *g.borrow_mut() = Some(closure);
        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    fn draw_console(state: &mut PyroConsoleState) {
        let len = state.lines.len();
        let max = cmp::min(len, state.offset as usize + LINES_VISIBLE);
        let range = cmp::min(state.offset as usize, max)..max;
        let lines = &state.lines[range];
        let (width, height) =
            (state.canvas.width() as f64, state.canvas.height() as f64);
        state.ctx.set_fill_style_str("white");
        state
            .ctx
            .set_font(format!("{}px monospace", FONT_SIZE).as_str());
        state.ctx.fill_rect(0.0, 0.0, width, height);
        state.ctx.set_fill_style_str("black");
        state.current_fill_style = "black".to_owned();
        // search through raw_lines for query
        for (i, line) in lines.iter().enumerate() {
            let mut style = "black".to_owned();
            let mut x = 0.0;

            for command in line.ansi_commands.iter() {
                match command {
                    AnsiCommand::RenderText(text) => {
                        if style != state.current_fill_style {
                            state.ctx.set_fill_style_str(style.as_str());
                            state.current_fill_style = style.clone();
                        }
                        state
                            .ctx
                            .fill_text(
                                text.as_str(),
                                (LINE_HEIGHT / 2) as f64 + x,
                                (i as f64 + 1.0) * LINE_HEIGHT as f64
                                    + LINE_OFFSET as f64,
                            )
                            .expect("failed to draw");
                        // x += state.ctx.measure_text(text.as_str()).unwrap().width();
                        // x += state.char_width * text.len() as f64;
                        x += state
                            .measure_cache
                            .measure(FONT_SIZE, text.as_str());
                    }
                    AnsiCommand::ModifyStyle(control) => {
                        style = control.to_color();
                    }
                    AnsiCommand::Reset => {
                        style = "black".to_owned();
                    }
                }
            }
        }

        state.ctx.set_font("12px monospace");
        state.ctx.set_fill_style_str("rgba(0, 0, 0, 0.5)");

        // ctx.fill_text(format!("FPS: {:.2}", fps).as_str(), width - 100.0, 20.0)
        //     .expect("failed to draw");
        let str =
            format!("FPS: {:.2} | Lines: {}", state.fps, state.lines.len());
        // let text_width = state.ctx.measure_text(str.as_str()).unwrap().width();
        // let text_width = state.char_width * str.len() as f64;
        let text_width = state.measure_cache.measure(12, str.as_str());
        state
            .ctx
            .fill_text(
                str.as_str(),
                width - text_width - 8.0 - SCROLL_BAR_WIDTH as f64,
                15.0,
            )
            .expect("failed to draw");

        // scroll bar

        let total_lines =
            cmp::max(len as u64, LINES_VISIBLE as u64) - LINES_VISIBLE as u64;

        state.ctx.set_fill_style_str("rgba(0, 0, 0, 0.15)");
        state.ctx.fill_rect(
            width - SCROLL_BAR_WIDTH as f64,
            0.0,
            SCROLL_BAR_WIDTH as f64,
            height,
        );

        let total_height = (LINE_HEIGHT as f64) * total_lines as f64;
        let scroll_bar_height = if total_lines <= LINES_VISIBLE as u64 {
            height as u64
        } else {
            cmp::max(
                (height / total_height * height).round() as u64,
                MIN_SCROLL_BAR_HEIGHT as u64,
            )
        };
        let scroll_bar_y = (height - scroll_bar_height as f64)
            * state.offset as f64
            / total_lines as f64;
        state.ctx.fill_rect(
            width - SCROLL_BAR_WIDTH as f64,
            scroll_bar_y as f64,
            SCROLL_BAR_WIDTH as f64,
            scroll_bar_height as f64,
        );
    }

    pub fn add_line(&mut self, line: &str) {
        let mut state = self.state.borrow_mut();
        let is_at_bottom = state.offset + LINES_VISIBLE as u64
            >= (state.lines.len() + GAP_LINES) as u64;
        let result = AnsiParser::parse(line);
        // state.lines = PyroConsole::calculate_line_breaks(&state);
        let new_lines = PyroConsole::calculate_line_breaks(&mut state, &result);
        for line in new_lines.clone() {
            state.lines.push(PyroConsoleLine {
                ansi_commands: line,
                raw_line: result.clone(),
            });
        }

        // info!(
        //     "{}, {}",
        //     state.offset + LINES_VISIBLE as u64,
        //     (state.lines.len() + GAP_LINES as usize) as u64
        // );

        if is_at_bottom {
            state.offset = (state.lines.len() + GAP_LINES)
                .saturating_sub(LINES_VISIBLE)
                as u64;
        }
    }

    fn calculate_line_breaks(
        state: &mut PyroConsoleState,
        line: &[AnsiCommand],
    ) -> Vec<Vec<AnsiCommand>> {
        let canvas_space =
            state.canvas.width() as f64 - SCROLL_BAR_WIDTH as f64 - 16.0;
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
                            state.measure_cache.measure(FONT_SIZE, word);
                        if x + width > canvas_space {
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

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    pub fn destroy(&self) {
        cancel_animation_frame(self.state.borrow().animation_frame);
    }

    fn calculate_offset(y: f64, height: f64, total_lines: u64) -> u64 {
        let total_height = LINE_HEIGHT as f64 * total_lines as f64; // total scrollable content height
        let scroll_ratio = height / total_height; // proportion of the visible area to the total content height
        let scroll_bar_height =
            f64::max(height * scroll_ratio, MIN_SCROLL_BAR_HEIGHT as f64); // actual scrollbar height
        let scroll_range = height - scroll_bar_height; // the scrollable range of the scrollbar
        let offset = ((y / scroll_range).clamp(0.0, 1.0)) * total_lines as f64;
        (offset.round() as u64).clamp(0, total_lines)
    }

    pub fn clear(&mut self) {
        let mut state = self.state.borrow_mut();
        state.offset = 0;
        state.lines.clear();
    }

    pub fn redraw(&mut self) {
        let mut state = self.state.borrow_mut();
        // state.lines = PyroConsole::calculate_line_breaks(&state);
        PyroConsole::check_for_resize(&mut state);
        PyroConsole::draw_console(&mut state);
    }

    pub fn check_for_resize(state: &mut PyroConsoleState) {
        let size = (state.canvas.width(), state.canvas.height());
        if size != state.last_size {
            state.last_size = size;
            state.lines.clear();

            let raw_lines: Vec<Vec<AnsiCommand>> =
                state.lines.iter().map(|pcl| pcl.raw_line.clone()).collect();

            for line in raw_lines.iter() {
                let is_at_bottom = state.lines.len() >= LINES_VISIBLE
                    && state.offset
                        >= state.lines.len().saturating_sub(LINES_VISIBLE)
                            as u64;

                let new_lines = PyroConsole::calculate_line_breaks(state, line);
                for line in new_lines {
                    state.lines.push(PyroConsoleLine {
                        ansi_commands: line.clone(),
                        raw_line: line,
                    });
                }

                if is_at_bottom {
                    state.offset = (state.lines.len() + GAP_LINES)
                        .saturating_sub(LINES_VISIBLE)
                        as u64;
                }
            }
        }
    }

    pub fn get_scroll_px(&self) -> f64 {
        let state = self.state.borrow();
        (state.offset * LINE_HEIGHT as u64) as f64
    }

    pub fn get_content_height(&self) -> u32 {
        // get the content height, where the content height is equal to get_scroll_px() when scrolled to the bottom
        let state = self.state.borrow();
        let total_lines =
            cmp::max(state.lines.len() as u64, LINES_VISIBLE as u64)
                - LINES_VISIBLE as u64;
        let total_height =
            (LINE_HEIGHT as f64) * (total_lines + GAP_LINES as u64) as f64;
        total_height as u32
    }

    pub fn mouse_down(
        &mut self,
        x: f64,
        y: f64,
        client_width: u32,
        client_height: u32,
    ) {
        let mut state = self.state.borrow_mut();
        let total_lines = (state.lines.len() - LINES_VISIBLE) as u64;
        let height = state.canvas.height() as f64;
        let total_height = (LINE_HEIGHT as f64) * total_lines as f64;
        let scroll_bar_height = cmp::max(
            (height / total_height * height).round() as u64,
            MIN_SCROLL_BAR_HEIGHT as u64,
        );
        let scroll_bar_y = (height - scroll_bar_height as f64)
            * state.offset as f64
            / total_lines as f64;

        // if the mouse isn't inside the scroll bar, return
        // scrollbar width should be accounted for
        if x < client_width as f64 - SCROLL_BAR_WIDTH as f64 {
            state.scroll_bar_y_offset = -1.0;
            return;
        }
        if y < scroll_bar_y || y > scroll_bar_y + scroll_bar_height as f64 {
            state.scroll_bar_y_offset = (scroll_bar_height as f64) / 2.0;
            state.offset = PyroConsole::calculate_offset(
                y - scroll_bar_height as f64 / 2.0,
                client_height as f64,
                (state.lines.len() - LINES_VISIBLE + GAP_LINES) as u64,
            );
        } else {
            state.scroll_bar_y_offset = y - scroll_bar_y;
        }
    }

    pub fn mouse_move(&mut self, y: f64, client_height: u32) {
        let mut state = self.state.borrow_mut();
        if state.scroll_bar_y_offset < 0.0 {
            return;
        }
        let y = y - state.scroll_bar_y_offset;

        let offset = PyroConsole::calculate_offset(
            y,
            client_height as f64,
            (state.lines.len() - LINES_VISIBLE + GAP_LINES) as u64,
        );
        state.offset = offset;
    }

    pub fn mouse_up(&mut self) {
        let mut state = self.state.borrow_mut();
        state.scroll_bar_y_offset = -1.0;
    }

    pub fn wheel(&mut self, delta_y: f64) {
        let mut state = self.state.borrow_mut();
        if delta_y > 0.0 {
            if state.offset == u64::MAX
                || state.offset as usize + LINES_VISIBLE
                    >= state.lines.len() + GAP_LINES
            {
                return;
            }
            state.offset += 1;
        } else {
            if state.offset == 0 {
                return;
            }
            state.offset -= 1;
        }
    }
}
