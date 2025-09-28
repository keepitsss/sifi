use super::*;
pub struct Article<'re, IsWithChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Article<'re, WithChild>>,
    pub(crate) has_child: PhantomData<IsWithChild>,
}
impl BuiltinHtmlElement for Article<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Article<'re, WithChild>);
impl FlowContent for Article<'_, WithChild> {}
impl SectioningContent for Article<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for Article<'_, WithChild> {}
impl<'re, IsWithChild> Article<'re, IsWithChild> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Article<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Article {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        Article {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
}
impl<'re> SimpleElement<'re> for Article<'re, WithChild> {
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
            name: self.arena.alloc("article"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
