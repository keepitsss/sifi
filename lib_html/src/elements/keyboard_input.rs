use super::*;
pub struct KeyboardInput<'re, HasChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, KeyboardInput<'re, WithChild>>,
    pub(crate) has_child: PhantomData<HasChild>,
}
impl BuiltinHtmlElement for KeyboardInput<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, KeyboardInput<'re, WithChild>);
impl FlowContent for KeyboardInput<'_, WithChild> {}
impl PhrasingContent for KeyboardInput<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for KeyboardInput<'_, WithChild> {}
impl<'re, HasChild> KeyboardInput<'re, HasChild> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> KeyboardInput<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let KeyboardInput {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        KeyboardInput {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
}
impl<'re> SimpleElement<'re> for KeyboardInput<'re, WithChild> {
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
            name: self.arena.alloc("kbd"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
