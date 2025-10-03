use super::*;
pub struct Link<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    // TODO: store Url
    pub href: Option<&'re str>,
    pub download: bool,
    pub ping: Vec<'re, &'re str>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Link<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Link<'re>);
impl FlowContent for Link<'_> {}
impl PhrasingContent for Link<'_> {}
impl<'re> Link<'re> {
    // FIXME: content model should be transparent
    /// # Safety
    /// Must be no interactive content descendant, a element descendant, or descendant with the tabindex attribute specified.
    ///
    /// Content model is transparent. ( You should be able to pass child directly to link parent )
    pub unsafe fn child(mut self, child: impl Renderable + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
    pub fn href(mut self, url: &str) -> Self {
        let url = url.trim();
        assert!(self.href.is_none());
        assert!(!url.contains(" "));
        self.href = Some(self.arena.alloc_str(url));
        self
    }
    pub fn download(mut self) -> Self {
        assert!(!self.download);
        self.download = true;
        self
    }
    pub fn ping(mut self, url: &str) -> Self {
        let url = url.trim();
        assert!(!url.contains(" "));
        assert!(!self.ping.contains(&url));
        self.ping.push(self.arena.alloc_str(url));
        self
    }
}
impl<'re> SimpleElement<'re> for Link<'re> {
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
        if let Some(href) = self.href {
            attrs.push(HtmlAttribute {
                name: "href",
                value: HtmlValue::String(href),
            });
        }
        if self.download {
            attrs.push(HtmlAttribute {
                name: "download",
                value: HtmlValue::Bool,
            })
        }
        if !self.ping.is_empty() {
            let mut value = bumpalo::collections::String::new_in(self.arena);
            value.push_str(self.ping[0]);
            for url in self.ping.iter().skip(1) {
                value.push(' ');
                value.push_str(url);
            }
            attrs.push(HtmlAttribute {
                name: "ping",
                value: HtmlValue::String(value.into_bump_str()),
            })
        }
        GenericHtmlElement {
            name: self.arena.alloc("a"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
