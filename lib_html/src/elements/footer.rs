use super::*;
pub struct Footer<'re, IsWithChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Footer<'re, WithChild>>,
    pub(crate) has_child: PhantomData<IsWithChild>,
}
impl BuiltinHtmlElement for Footer<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Footer<'re, WithChild>);
impl FlowContent for Footer<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for Footer<'_, WithChild> {}
impl<'re, HasChild> Footer<'re, HasChild> {
    /// # Safety
    /// Must be no header or footer element descendants.
    pub unsafe fn child(mut self, child: impl FlowContent + 're) -> Footer<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Footer {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        Footer {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
}
impl<'re> SimpleElement<'re> for Footer<'re, WithChild> {
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
            name: self.arena.alloc("footer"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
