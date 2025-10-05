use super::*;
pub struct Heading<'re, IsWithChild> {
    pub level: u8,
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Heading<'re, WithChild>>,
    pub(crate) has_child: PhantomData<IsWithChild>,
}
impl BuiltinHtmlElement for Heading<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Heading<'re, WithChild>);
impl FlowContent for Heading<'_, WithChild> {}
impl HeadingContent for Heading<'_, WithChild> {}
// # Safety
// Typesafe design.
unsafe impl PalpableContent for Heading<'_, WithChild> {}
impl<'re, IsWithChild> Heading<'re, IsWithChild> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Heading<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Heading {
            level,
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        Heading {
            level,
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
}
impl<'re> SimpleElement<'re> for Heading<'re, WithChild> {
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
            name: self.arena.alloc(format!("h{}", self.level)),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
