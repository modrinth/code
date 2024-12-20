use std::collections::HashMap;

use web_sys::OffscreenCanvasRenderingContext2d;

/**
  A cache for measuring text width.

  This brings huge performance improvements because of the overhead when calling JS functions.
*/
#[derive(Debug, Clone)]
pub struct TextMeasureCache {
    // font_size -> width
    cache: HashMap<u32, f64>,
    ctx: OffscreenCanvasRenderingContext2d,
}

impl TextMeasureCache {
    pub fn new(ctx: OffscreenCanvasRenderingContext2d) -> Self {
        TextMeasureCache {
            cache: HashMap::new(),
            ctx,
        }
    }

    pub fn measure(&mut self, font_size: usize, text: &str) -> f64 {
        let font_size = font_size as u32;
        if let Some(width) = self.cache.get(&font_size) {
            return *width * text.len() as f64;
        }

        let char_width = self.measure_char(font_size, "a");
        let width = char_width * text.len() as f64;
        self.cache.insert(font_size, char_width);
        width
    }

    fn measure_char(&self, font_size: u32, text: &str) -> f64 {
        self.ctx.set_font(&format!("{}px monospace", font_size));
        self.ctx.measure_text(text).unwrap().width()
    }
}
