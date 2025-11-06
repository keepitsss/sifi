use super::*;
pub struct Paragraph<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Paragraph<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Paragraph<'re>);
impl FlowContent for Paragraph<'_> {}
impl<'re> Paragraph<'re> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Paragraph<'re> {
    type GenericSelf = Self;
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut attrs = Vec::new_in(self.arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: self.arena.alloc("p"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
