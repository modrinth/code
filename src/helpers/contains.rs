use handlebars::*;

#[derive(Clone, Copy)]
pub struct ContainsHelper;

impl HelperDef for ContainsHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'_>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let array = h
            .param(0)
            .map(|v| serde_json::from_value::<Vec<String>>(v.value().clone()).unwrap())
            .ok_or_else(|| RenderError::new("Parameter not found!"))?;
        let value = h
            .param(1)
            .map(|v| v.value().as_str().unwrap())
            .ok_or_else(|| RenderError::new("Parameter not found!"))?;

        let tmpl = if array.contains(&String::from(value)) {
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
