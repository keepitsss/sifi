use super::*;

#[derive(Clone)]
pub struct Html<'re> {
    pub(crate) head: Head<'re>,
    pub(crate) body: Option<Body<'re>>,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
    pub(crate) arena: &'re Bump,
}
derive_pre_render_hooks!('re, Html<'re>);
impl<'re> Html<'re> {
    pub fn body(&mut self, body: Body<'re>) {
        assert!(self.body.is_none());
        self.body = Some(body);
    }
}
impl<'re> SimpleElement<'re> for Html<'re> {
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        GenericHtmlElement {
            name: "html",
            attributes: &[],
            children:
                bumpalo::vec![in self.arena; self.arena.alloc(self.body.clone().expect("body should be set")) as &dyn Renderable]
                    .into_bump_slice(),
                late_children: bumpalo::vec![in self.arena; self.arena.alloc(self.head.clone()) as &dyn Renderable].into_bump_slice(),
        }
    }
}
#[derive(Clone)]
pub struct Head<'re> {
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
    pub(crate) arena: &'re Bump,
}
derive_pre_render_hooks!('re, Head<'re>);
impl<'re> SimpleElement<'re> for Head<'re> {
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut children = Vec::new_in(self.arena);
        children.push(self.arena.alloc(GlobalStyles) as &dyn Renderable);
        GenericHtmlElement {
            name: "head",
            attributes: &[],
            children: children.into_bump_slice(),
            late_children: &[],
        }
    }
}

struct GlobalStyles;
impl Renderable for GlobalStyles {
    fn render(&self, cx: &mut Context) {
        let mut styles = Vec::new_in(cx.arena);
        styles.extend(cx.styles.iter());
        Style(styles).render(cx)
    }
}
struct Style<'re>(Vec<'re, &'re str>);
impl<'re> Renderable for Style<'re> {
    fn render(&self, cx: &mut Context) {
        cx_writeln!(cx, "{}<style>", cx.indentation);
        cx.indentation.level += 1;
        cx_writeln!(cx, "");
        for i in 0..self.0.len() {
            cx_writeln!(cx, "{}{}", /* cx.indentation */ "", self.0[i].trim());
            cx_writeln!(cx, "");
        }
        cx.indentation.level -= 1;
        cx_writeln!(cx, "{}</style>", cx.indentation);
    }
}

impl Renderable for &str {
    fn render(&self, cx: &mut Context) {
        writeln!(cx.output, "{}{self}", cx.indentation).unwrap();
    }
}
impl Renderable for &mut str {
    fn render(&self, cx: &mut Context) {
        writeln!(cx.output, "{}{self}", cx.indentation).unwrap();
    }
}
impl FlowContent for &str {}
impl PhrasingContent for &str {}
impl FlowContent for &mut str {}
impl PhrasingContent for &mut str {}

#[derive(Clone)]
pub struct Body<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
    pub(crate) arena: &'re Bump,
}
impl<'re> Body<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl BuiltinHtmlElement for Body<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Body<'re>);
impl<'re> SimpleElement<'re> for Body<'re> {
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut attrs = Vec::new_in(self.arena);
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: "body",
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

#[derive(Clone, Copy)]
pub enum HtmlValue<'re> {
    Number(u32),
    String(&'re str),
    Bool(bool),
    Empty,
}
#[derive(Clone, Copy)]
pub struct HtmlAttribute<'re> {
    name: &'re str,
    value: HtmlValue<'re>,
}
impl<'re> Renderable for HtmlAttribute<'re> {
    fn render(&self, cx: &mut Context) {
        let name = &self.name;
        match &self.value {
            HtmlValue::Number(number) => cx_write!(cx, " {name}={number}"),
            HtmlValue::String(string) => cx_write!(cx, " {name}=\"{string}\""), // FIXME: escaping
            HtmlValue::Bool(bool) => cx_write!(cx, " {name}={bool}"),
            HtmlValue::Empty => cx_write!(cx, " {name}"),
        }
    }
}

#[derive(Clone)]
pub struct GenericHtmlElement<'re> {
    pub name: &'re str,
    pub attributes: &'re [HtmlAttribute<'re>],
    pub children: &'re [&'re dyn Renderable],
    /// It will render after children, but displayed before.
    pub late_children: &'re [&'re dyn Renderable],
}
impl<'re> Renderable for GenericHtmlElement<'re> {
    fn render(&self, cx: &mut Context) {
        cx_write!(cx, "{}<{}", cx.indentation, self.name);
        for attribute in self.attributes {
            attribute.render(cx);
        }
        if self.children.is_empty() {
            cx_writeln!(cx, "/>");
            return;
        }
        cx_writeln!(cx, ">");

        let mut output_buf = String::new();
        std::mem::swap(&mut cx.output, &mut output_buf);

        for child in self.children {
            cx.indentation.level += 1;
            child.render(&mut *cx);
            cx.indentation.level -= 1;
        }
        std::mem::swap(&mut cx.output, &mut output_buf);

        for child in self.late_children {
            cx.indentation.level += 1;
            child.render(&mut *cx);
            cx.indentation.level -= 1;
        }
        cx.output += &output_buf;

        cx_writeln!(cx, "{}</{}>", cx.indentation, self.name);
    }
}

pub trait SimpleElement<'re>: PreRenderHooks<'re> {
    #[allow(clippy::wrong_self_convention)]
    /// # Safety
    /// You should call pre_render_hooks before rendering
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re>;
}
fn strip_anyelement<'re>(
    arena: &'re Bump,
    children: &Vec<'re, AnyElement<'re>>,
) -> &'re [&'re dyn Renderable] {
    let mut children_clone = Vec::new_in(arena);
    children_clone.extend(children.into_iter().map(|x| x.0));
    children_clone.into_bump_slice()
}
impl<'re, T> Renderable for T
where
    T: SimpleElement<'re, This = Self>,
{
    fn render(&self, cx: &mut Context) {
        if let Some(hook) = self.get_pre_render_hook() {
            hook(self, cx);
        }
        // SAFETY: hook called
        unsafe {
            self.into_html_element().render(cx);
        }
    }
}

type Hook<'re, This> = &'re dyn Fn(&This, &mut Context);
pub trait PreRenderHooks<'re>: Sized
where
    Self: 're,
{
    type This;
    /// # Safety
    /// This will overwrite previous hook
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're);

    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>>;

    fn with_pre_render_hook(mut self, new_hook: impl Fn(&Self::This, &mut Context) + 're) -> Self {
        self.add_pre_render_hook(new_hook);
        self
    }
    fn add_pre_render_hook(&mut self, new_hook: impl Fn(&Self::This, &mut Context) + 're) {
        match self.get_pre_render_hook() {
            Some(prev_hook) => unsafe {
                self.set_pre_render_hook(move |this, cx| {
                    prev_hook(this, cx);
                    new_hook(this, cx);
                });
            },
            None => unsafe {
                self.set_pre_render_hook(new_hook);
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PreRenderHookStorage<'re, This> {
    hook: Option<Hook<'re, This>>,
    arena: &'re Bump,
}
impl<'re, T> PreRenderHookStorage<'re, T> {
    pub(crate) fn new_in(arena: &'re Bump) -> Self {
        PreRenderHookStorage { hook: None, arena }
    }
}
impl<'re, This> PreRenderHooks<'re> for PreRenderHookStorage<'re, This> {
    type This = This;
    fn get_pre_render_hook(&self) -> Option<Hook<'re, This>> {
        self.hook
    }
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&This, &mut Context) + 're) {
        let hook = self.arena.alloc(hook) as Hook<'re, This>;
        self.hook = Some(self.arena.alloc(|this: &This, cx: &mut Context| {
            hook(this, cx);
        }));
    }
}

pub trait Attribute<'re> {
    fn new_in(arena: &'re Bump) -> Self;
    fn render(&self) -> Option<HtmlAttribute<'re>>;
}
#[derive(Clone)]
pub struct Classes<'re>(pub Vec<'re, &'re str>);
impl<'re> Attribute<'re> for Classes<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Self(Vec::new_in(arena))
    }
    fn render(&self) -> Option<HtmlAttribute<'re>> {
        if self.0.is_empty() {
            None
        } else {
            Some(HtmlAttribute {
                name: "class",
                value: HtmlValue::String(self.0.bump().alloc_str(&self.0.join(" "))),
            })
        }
    }
}
impl<'re> Classes<'re> {
    pub fn add(&mut self, class: &str) {
        assert!(!self.0.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!class.is_empty());
        self.0.push(self.0.bump().alloc_str(class));
    }
}

pub struct Div<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Div<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Div<'re>);
impl FlowContent for Div<'_> {}
impl<'re> Div<'re> {
    pub fn child(mut self, child: impl Renderable + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Div<'re> {
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
            name: self.arena.alloc("div"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

pub struct Heading<'re> {
    pub level: u8,
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Heading<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Heading<'re>);
impl FlowContent for Heading<'_> {}
impl HeadingContent for Heading<'_> {}
impl<'re> Heading<'re> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Heading<'re> {
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

pub struct Paragraph<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Paragraph<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Paragraph<'re>);
impl FlowContent for Paragraph<'_> {}
impl<'re> Paragraph<'re> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Paragraph<'re> {
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
            name: self.arena.alloc("p"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

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
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
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
                value: HtmlValue::Empty,
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

pub struct Article<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Article<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Article<'re>);
impl FlowContent for Article<'_> {}
impl SectioningContent for Article<'_> {}
// # Safety
// Pre render hook added on creation to check that article has at least on child.
unsafe impl PalpableConent for Article<'_> {}
impl<'re> Article<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Article<'re> {
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

pub struct Section<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Section<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Section<'re>);
impl FlowContent for Section<'_> {}
impl SectioningContent for Section<'_> {}
// # Safety
// Pre render hook added on creation to check that article has at least on child.
unsafe impl PalpableConent for Section<'_> {}
impl<'re> Section<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Section<'re> {
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
            name: self.arena.alloc("section"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

pub struct Navigation<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Navigation<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Navigation<'re>);
impl FlowContent for Navigation<'_> {}
impl SectioningContent for Navigation<'_> {}
// # Safety
// Pre render hook added on creation to check that article has at least on child.
unsafe impl PalpableConent for Navigation<'_> {}
impl<'re> Navigation<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Navigation<'re> {
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
            name: self.arena.alloc("nav"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

pub struct Aside<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Aside<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, Aside<'re>);
impl FlowContent for Aside<'_> {}
impl SectioningContent for Aside<'_> {}
// # Safety
// Pre render hook added on creation to check that article has at least on child.
unsafe impl PalpableConent for Aside<'_> {}
impl<'re> Aside<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Aside<'re> {
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
            name: self.arena.alloc("aside"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
