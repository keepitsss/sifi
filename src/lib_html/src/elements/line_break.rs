use super::*;
pub struct LineBreak<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, LineBreak<'re>>,
}
impl BuiltinHtmlElement for LineBreak<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, LineBreak<'re>);
impl FlowContent for LineBreak<'_> {}
impl PhrasingContent for LineBreak<'_> {}
impl<'re> SimpleElement<'re> for LineBreak<'re> {
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
            name: self.arena.alloc("br"),
            attributes: attrs.into_bump_slice(),
            children: &[],
            late_children: &[],
        }
    }
}
