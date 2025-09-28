use super::*;
pub struct WithoutHeader;
pub struct WithHeader;
pub struct HeadingGroup<'re, IsWithHeading> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, HeadingGroup<'re, WithHeader>>,
    pub(crate) has_heading: PhantomData<IsWithHeading>,
}
impl BuiltinHtmlElement for HeadingGroup<'_, WithHeader> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, HeadingGroup<'re, WithHeader>);
impl FlowContent for HeadingGroup<'_, WithHeader> {}
impl HeadingContent for HeadingGroup<'_, WithHeader> {}
// # Safety
// Typesafe design.
unsafe impl PalpableContent for HeadingGroup<'_, WithHeader> {}
impl<'re, IsWithHeading> HeadingGroup<'re, IsWithHeading> {
    pub fn p(mut self, child: Paragraph<'re>) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> HeadingGroup<'re, WithoutHeader> {
    pub fn heading(mut self, child: Heading<'re, WithChild>) -> HeadingGroup<'re, WithHeader> {
        self.children.push(child.into_any_element(self.arena));
        let HeadingGroup {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_heading: _,
        } = self;
        HeadingGroup {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_heading: PhantomData,
        }
    }
}
impl<'re> SimpleElement<'re> for HeadingGroup<'re, WithHeader> {
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
            name: self.arena.alloc("hgroup"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
