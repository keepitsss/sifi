use super::*;
pub struct TableFooter<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, TableFooter<'re>>,
}
impl BuiltinHtmlElement for TableFooter<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, TableFooter<'re>);
impl<'re> TableFooter<'re> {
    pub fn child(mut self, child: TableRow<'re>) -> TableFooter<'re> {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for TableFooter<'re> {
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
            name: self.arena.alloc("tfoot"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
