mod contains;
mod format_human;
use handlebars::*;

pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("contains", Box::new(contains::ContainsHelper));
    handlebars.register_helper("format", Box::new(format_human::HumanFormatHelper));
}
