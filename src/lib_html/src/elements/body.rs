use super::*;
#[derive(Clone)]
pub struct Body<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
    pub(crate) arena: &'re Bump,
}
impl<'re> Body<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl BuiltinHtmlElement for Body<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Body<'re>);
impl<'re> SimpleElement<'re> for Body<'re> {
    type GenericSelf = Self;
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut attrs = Vec::new_in(self.arena);
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: "body",
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
