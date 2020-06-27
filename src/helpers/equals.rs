use handlebars::*;

#[derive(Clone, Copy)]
pub struct EqualsHelper;

impl HelperDef for EqualsHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'_>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let a = h
            .param(0)
            .map(|v| v.value().as_object().unwrap())
            .ok_or_else(|| RenderError::new("Parameter not found!"))?;

        let b = h
            .param(1)
            .map(|v| v.value().as_object().unwrap())
            .ok_or_else(|| RenderError::new("Parameter not found!"))?;

        let tmpl = if a == b {
            h.template()
        } else {
            h.inverse()
        };

        match tmpl {
            Some(ref t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}
