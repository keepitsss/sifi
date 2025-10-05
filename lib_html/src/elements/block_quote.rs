use super::*;
pub struct BlockQuote<'re, HasChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    // TODO: should be url
    pub cite: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, BlockQuote<'re, WithChild>>,
    pub(crate) has_child: PhantomData<HasChild>,
}
impl BuiltinHtmlElement for BlockQuote<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, BlockQuote<'re, WithChild>);
impl FlowContent for BlockQuote<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for BlockQuote<'_, WithChild> {}
impl<'re, HasChild> BlockQuote<'re, HasChild> {
    pub fn child(mut self, child: impl FlowContent + 're) -> BlockQuote<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let BlockQuote {
            classes,
            id,
            cite,
            children,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        BlockQuote {
            classes,
            id,
            cite,
            children,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }

    /// If the `cite` attribute is present, it must be a valid URL potentially surrounded by spaces.
    /// User agents may allow users to follow such citation links, but they are primarily intended for private use
    /// (e.g., by server-side scripts collecting statistics about a site's use of quotations), not for readers.
    pub fn cite(mut self, cite: &str) -> Self {
        assert!(self.cite.is_none());
        assert!(!cite.trim().is_empty());
        self.cite = Some(self.arena.alloc_str(cite));
        self
    }
}
impl<'re> SimpleElement<'re> for BlockQuote<'re, WithChild> {
    type GenericSelf = Self;
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut attrs = Vec::new_in(self.arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if let Some(cite) = self.cite {
            attrs.push(HtmlAttribute {
                name: "cite",
                value: HtmlValue::String(cite),
            })
        }
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: self.arena.alloc("blockquote"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
