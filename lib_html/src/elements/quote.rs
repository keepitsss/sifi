use super::*;

pub struct CiteAttribute<'re> {
    source_address: Option<&'re str>,
}
impl<'re> Attribute<'re> for CiteAttribute<'re> {
    fn new_in(_arena: &'re Bump) -> Self {
        CiteAttribute {
            source_address: None,
        }
    }

    fn render(&self) -> Option<HtmlAttribute<'re>> {
        self.source_address.map(|address| HtmlAttribute {
            name: "cite",
            value: HtmlValue::String(address),
        })
    }
}

pub struct Quote<'re, HasChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub cite: CiteAttribute<'re>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Quote<'re, WithChild>>,
    pub(crate) has_child: PhantomData<HasChild>,
}
impl BuiltinHtmlElement for Quote<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Quote<'re, WithChild>);
impl FlowContent for Quote<'_, WithChild> {}
impl PhrasingContent for Quote<'_, WithChild> {}
// # Safety
// Typesafe design
unsafe impl PalpableContent for Quote<'_, WithChild> {}
impl<'re, HasChild> Quote<'re, HasChild> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Quote<'re, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Quote {
            classes,
            id,
            children,
            cite,
            arena,
            pre_render_hook,
            has_child: _,
        } = self;
        Quote {
            classes,
            id,
            children,
            cite,
            arena,
            pre_render_hook,
            has_child: PhantomData,
        }
    }
    pub fn cite(mut self, source: &str) -> Self {
        assert!(self.cite.source_address.is_none());
        self.cite.source_address = Some(self.arena.alloc_str(source));
        self
    }
}
impl<'re> SimpleElement<'re> for Quote<'re, WithChild> {
    type GenericSelf = Self;
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut attrs = Vec::new_in(self.arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if let Some(attr) = self.cite.render() {
            attrs.push(attr);
        }
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: self.arena.alloc("q"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
