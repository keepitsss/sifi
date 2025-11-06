use super::*;
pub struct Code<'re, HasChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Code<'re, WithChild>>,
    pub(crate) has_child: PhantomData<HasChild>,
}
impl BuiltinHtmlElement for Code<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Code<'re, WithChild>);
impl FlowContent for Code<'_, WithChild> {}
impl PhrasingContent for Code<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for Code<'_, WithChild> {}
impl<'re, HasChild> Code<'re, HasChild> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Code<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Code {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        Code {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
}
impl<'re> Code<'re, WithChild> {
    /// # Safety
    /// ***There is no formal way to indicate the language of computer code being marked up.***
    ///
    /// Authors who wish to mark code elements with the language used, e.g. so that syntax highlighting scripts can use the right rules, can use the class attribute,
    ///     e.g. by adding a class prefixed with "language-" to the element.
    pub unsafe fn language(self, lang: &str) -> Self {
        self.class(&format!("language-{lang}"))
    }
}
impl<'re> SimpleElement<'re> for Code<'re, WithChild> {
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
            name: self.arena.alloc("code"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
