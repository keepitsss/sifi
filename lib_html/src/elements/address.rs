use super::*;
pub struct Address<'re, HasChild: ChildExistenceState> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Address<'re, WithChild>>,
    pub(crate) has_child: PhantomData<HasChild>,
}
impl BuiltinHtmlElement for Address<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Address<'re, WithChild>);
impl FlowContent for Address<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for Address<'_, WithChild> {}
impl<'re, HasChild: ChildExistenceState> Address<'re, HasChild> {
    /// # Safety
    /// Must be no heading content descendants, no sectioning content descendants, and no header, footer, or address element descendants.
    pub unsafe fn child(mut self, child: impl FlowContent + 're) -> Address<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Address {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        Address {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
}
impl<'re> SimpleElement<'re> for Address<'re, WithChild> {
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
            name: self.arena.alloc("address"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
