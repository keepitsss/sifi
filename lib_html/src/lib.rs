//! 're = 'rendering
//!
//! Idea: implement palpable content by initializing element with pre_render_callback check

use std::{collections::HashSet, fmt::Write};

use bumpalo::{Bump, collections::Vec};

macro_rules! cx_write {
    ($output:expr, $($arg:tt)*) => {
        write!($output.output, $($arg)+).unwrap()
    };
}
macro_rules! cx_writeln {
    ($output:expr, $($arg:tt)*) => {
        writeln!($output.output, $($arg)+).unwrap()
    };
}

macro_rules! derive_pre_render_hooks {
    ($lifetime:lifetime, $ty:ty) => {
        impl<$lifetime> PreRenderHooks<$lifetime> for $ty {
            type This = Self;
            unsafe fn set_pre_render_hook(
                &mut self,
                hook: impl Fn(&Self, &mut Context) + $lifetime,
            ) {
                unsafe {
                    self.pre_render_hook.set_pre_render_hook(hook);
                }
            }
            fn get_pre_render_hook(&self) -> Option<Hook<$lifetime, Self>> {
                self.pre_render_hook.get_pre_render_hook()
            }
        }
    };
}

pub mod utils;

pub trait Renderable {
    fn render(&self, cx: &mut Context);
}

pub struct Context<'re> {
    pub indentation: utils::Indentation,
    pub output: std::string::String,
    pub arena: &'re Bump,
    pub ids: HashSet<&'re str>,
    pub styles: HashSet<&'re str>,
}

#[derive(Clone, Copy)]
pub struct AnyElement<'re>(pub &'re dyn Renderable);
impl<'re> Renderable for AnyElement<'re> {
    fn render(&self, cx: &mut Context) {
        self.0.render(cx);
    }
}
pub trait IntoElement<'re> {
    fn into_any_element(self, arena: &'re Bump) -> AnyElement<'re>;
}
impl<'re, T> IntoElement<'re> for T
where
    T: Renderable + 're,
{
    fn into_any_element(self, arena: &'re Bump) -> AnyElement<'re> {
        AnyElement(arena.alloc(self) as &dyn Renderable)
    }
}
/// https://html.spec.whatwg.org/#flow-content-2
pub trait FlowContent: Renderable {}
pub trait SectioningContent: FlowContent {}
pub trait HeadingContent: FlowContent {}
pub trait PhrasingContent: FlowContent {}
pub trait EmbeddedContent: PhrasingContent {}
pub trait InteractiveContent: FlowContent {}
pub trait MetadataContent: Renderable {}
/// # Safety
/// see docs, TLDR: should have content
pub unsafe trait PalpableConent: FlowContent {}
pub trait SelectInnerConent: Renderable {}
pub trait OptgroupInnerConent: Renderable {}
pub trait OptionInnerConent: Renderable {}

pub trait BuiltinHtmlElement: Sized {
    fn class(self, class: &str) -> Self;
    fn classes<'a>(mut self, classes: impl IntoIterator<Item = &'a str>) -> Self {
        let mut count = 0;
        for class in classes.into_iter() {
            self = self.class(class);
            count += 1;
        }
        assert_ne!(count, 0, "empty classes provided");
        assert_ne!(count, 1, "use 'class' method to provide one class");
        self
    }
    fn id(self, id: &str) -> Self;
}

#[derive(Clone)]
pub struct Html<'re> {
    head: Head<'re>,
    body: Option<Body<'re>>,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
    arena: &'re Bump,
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
    pre_render_hook: PreRenderHookStorage<'re, Self>,
    arena: &'re Bump,
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
    pre_render_hook: PreRenderHookStorage<'re, Self>,
    arena: &'re Bump,
}
impl<'re> Body<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl BuiltinHtmlElement for Body<'_> {
    fn class(mut self, class: &str) -> Self {
        self.classes.add(class);
        self
    }
    fn id(mut self, id: &str) -> Self {
        assert!(self.id.is_none());
        assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!id.is_empty());
        self.id = Some(self.arena.alloc_str(id));
        self.with_pre_render_hook(|this: &Self, cx: &mut Context| {
            let id = this.id.unwrap();
            if cx.ids.contains(id) {
                panic!("'{id}' id duplicate");
            }
            cx.ids.insert(cx.arena.alloc_str(id));
        })
    }
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
struct PreRenderHookStorage<'re, This> {
    hook: Option<Hook<'re, This>>,
    arena: &'re Bump,
}
impl<'re, T> PreRenderHookStorage<'re, T> {
    fn new_in(arena: &'re Bump) -> Self {
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
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Div<'_> {
    fn class(mut self, class: &str) -> Self {
        self.classes.add(class);
        self
    }
    fn id(mut self, id: &str) -> Self {
        assert!(self.id.is_none());
        assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!id.is_empty());
        self.id = Some(self.arena.alloc_str(id));
        self.with_pre_render_hook(|this: &Self, cx: &mut Context| {
            let id = this.id.unwrap();
            if cx.ids.contains(id) {
                panic!("'{id}' id duplicate");
            }
            cx.ids.insert(cx.arena.alloc_str(id));
        })
    }
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

pub struct Heading1<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Heading1<'_> {
    fn class(mut self, class: &str) -> Self {
        self.classes.add(class);
        self
    }
    fn id(mut self, id: &str) -> Self {
        assert!(self.id.is_none());
        assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!id.is_empty());
        self.id = Some(self.arena.alloc_str(id));
        self.with_pre_render_hook(|this: &Self, cx: &mut Context| {
            let id = this.id.unwrap();
            if cx.ids.contains(id) {
                panic!("'{id}' id duplicate");
            }
            cx.ids.insert(cx.arena.alloc_str(id));
        })
    }
}
derive_pre_render_hooks!('re, Heading1<'re>);
impl FlowContent for Heading1<'_> {}
impl HeadingContent for Heading1<'_> {}
impl<'re> Heading1<'re> {
    pub fn child(mut self, child: impl PhrasingContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Heading1<'re> {
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
            name: self.arena.alloc("h1"),
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
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Paragraph<'_> {
    fn class(mut self, class: &str) -> Self {
        self.classes.add(class);
        self
    }
    fn id(mut self, id: &str) -> Self {
        assert!(self.id.is_none());
        assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!id.is_empty());
        self.id = Some(self.arena.alloc_str(id));
        self.with_pre_render_hook(|this: &Self, cx: &mut Context| {
            let id = this.id.unwrap();
            if cx.ids.contains(id) {
                panic!("'{id}' id duplicate");
            }
            cx.ids.insert(cx.arena.alloc_str(id));
        })
    }
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
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Link<'_> {
    fn class(mut self, class: &str) -> Self {
        self.classes.add(class);
        self
    }
    fn id(mut self, id: &str) -> Self {
        assert!(self.id.is_none());
        assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!id.is_empty());
        self.id = Some(self.arena.alloc_str(id));
        self.with_pre_render_hook(|this: &Self, cx: &mut Context| {
            let id = this.id.unwrap();
            if cx.ids.contains(id) {
                panic!("'{id}' id duplicate");
            }
            cx.ids.insert(cx.arena.alloc_str(id));
        })
    }
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

pub fn html(arena: &Bump) -> Html<'_> {
    Html {
        head: head(arena),
        body: None,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        arena,
    }
}
fn head(arena: &Bump) -> Head<'_> {
    Head {
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        arena,
    }
}
pub fn body(arena: &Bump) -> Body<'_> {
    Body {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        arena,
    }
}

pub fn div(arena: &Bump) -> Div<'_> {
    Div {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
pub fn h1(arena: &Bump) -> Heading1<'_> {
    Heading1 {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
pub fn p(arena: &Bump) -> Paragraph<'_> {
    Paragraph {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
pub fn a(arena: &Bump) -> Link<'_> {
    Link {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        href: None,
        download: false,
        ping: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
