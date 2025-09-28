use super::*;
pub struct ThematicBreak<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for ThematicBreak<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, ThematicBreak<'re>);
impl FlowContent for ThematicBreak<'_> {}
impl<'re> SimpleElement<'re> for ThematicBreak<'re> {
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
            name: self.arena.alloc("hr"),
            attributes: attrs.into_bump_slice(),
            children: &[],
            late_children: &[],
        }
    }
}
