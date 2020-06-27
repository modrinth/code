mod contains;
mod format_human;
mod equals;

use handlebars::*;

pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("contains", Box::new(contains::ContainsHelper));
    handlebars.register_helper("format", Box::new(format_human::HumanFormatHelper));

    //This helper is not used yet, but could be useful in many circumstances
    handlebars.register_helper("equals", Box::new(equals::EqualsHelper));
}
