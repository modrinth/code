extern crate human_format;
use handlebars::*;

#[derive(Clone, Copy)]
pub struct HumanFormatHelper;

impl HelperDef for HumanFormatHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'_>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("0.0");

        if param.len() > 3 {
            let mut formatted = human_format::Formatter::new().format(param.parse().unwrap());
            formatted.retain(|c| !c.is_whitespace());

            out.write(formatted.to_uppercase().as_ref())?;
        } else {
            out.write(param)?;
        }

        Ok(())
    }
}
