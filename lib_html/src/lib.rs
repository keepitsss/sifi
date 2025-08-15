//! 're = 'rendering
//!
//! Idea: implement palpable content by initializing element with pre_render_callback check

use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fmt::Write,
};

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
                &self,
                hook: impl FnMut(&Self, &mut Context) + $lifetime,
            ) {
                unsafe {
                    self.pre_render_hook.set_pre_render_hook(hook);
                }
            }
            fn take_pre_render_hook(&self) -> Option<Hook<$lifetime, Self>> {
                self.pre_render_hook.take_pre_render_hook()
            }
        }
    };
}

pub mod utils;

pub trait Renderable<'re> {
    fn render(&self, cx: &mut Context<'re>);
}

pub struct Context<'re> {
    pub indentation: utils::Indentation,
    pub output: std::string::String,
    pub arena: &'re Bump,
    pub ids: HashSet<&'re str>,
    pub styles: HashSet<&'re str>,
}

#[derive(Clone, Copy)]
pub struct AnyElement<'re>(pub &'re dyn Renderable<'re>);
impl<'re> Renderable<'re> for AnyElement<'re> {
    fn render(&self, cx: &mut Context<'re>) {
        self.0.render(cx);
    }
}
pub trait Component<'re>: Renderable<'re> + Sized + 're {
    fn into_any_element(self, arena: &'re Bump) -> AnyElement<'re>;
}
impl<'re, T> Component<'re> for T
where
    T: Renderable<'re> + 're,
{
    fn into_any_element(self, arena: &'re Bump) -> AnyElement<'re> {
        let value = arena.alloc(self);
        AnyElement(value)
    }
}
/// https://html.spec.whatwg.org/#flow-content-2
pub trait FlowContent<'re>: Component<'re> {}
pub trait SectioningContent<'re>: FlowContent<'re> {}
pub trait HeadingContent<'re>: FlowContent<'re> {}
pub trait PhrasingContent<'re>: FlowContent<'re> {}
pub trait EmbeddedContent<'re>: PhrasingContent<'re> {}
pub trait InteractiveContent<'re>: FlowContent<'re> {}
pub trait MetadataContent<'re>: Component<'re> {}
/// # Safety
/// see docs, TLDR: should have content
pub unsafe trait PalpableConent<'re>: FlowContent<'re> {}
pub trait SelectInnerConent<'re>: Component<'re> {}
pub trait OptgroupInnerConent<'re>: Component<'re> {}
pub trait OptionInnerConent<'re>: Component<'re> {}

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
pub mod tailwind;

pub struct Html<'re> {
    head: &'re RefCell<Head<'re>>,
    body: &'re RefCell<Body<'re>>,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
derive_pre_render_hooks!('re, Html<'re>);
impl<'re> Html<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Html {
            head: arena.alloc(RefCell::new(Head::new_in(arena))),
            body: arena.alloc(RefCell::new(Body::new_in(arena))),
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        }
    }
    pub fn add_to_body(&mut self, body: impl FlowContent<'re> + 're) {
        let children = &mut self.body.borrow_mut().children;
        children.push(body.into_any_element(children.bump()));
    }
}
impl<'re> SimpleElement<'re> for Html<'re> {
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
    {
        GenericHtmlElement {
            name: "html",
            attributes: &[],
            children: bumpalo::vec![in arena; self.body as &dyn Renderable].into_bump_slice(),
            late_children: bumpalo::vec![in arena; self.head as &dyn Renderable].into_bump_slice(),
        }
    }
}
impl<'re, T> Renderable<'re> for RefCell<T>
where
    T: Renderable<'re>,
{
    fn render(&self, cx: &mut Context<'re>) {
        (*self.borrow()).render(cx);
    }
}
pub struct Head<'re> {
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl<'re> Head<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Head {
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        }
    }
}
derive_pre_render_hooks!('re, Head<'re>);
impl<'re> SimpleElement<'re> for Head<'re> {
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
    {
        let mut children = Vec::new_in(arena);
        children.push(arena.alloc(GlobalStyles) as &dyn Renderable);
        GenericHtmlElement {
            name: "head",
            attributes: &[],
            children: children.into_bump_slice(),
            late_children: &[],
        }
    }
}

struct GlobalStyles;
impl Renderable<'_> for GlobalStyles {
    fn render(&self, cx: &mut Context<'_>) {
        let mut styles = Vec::new_in(cx.arena);
        styles.extend(cx.styles.iter());
        Style(styles).render(cx)
    }
}
struct Style<'re>(Vec<'re, &'re str>);
impl<'re> Renderable<'re> for Style<'re> {
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

impl<'re> Renderable<'re> for &'re str {
    fn render(&self, cx: &mut Context) {
        writeln!(cx.output, "{}{self}", cx.indentation).unwrap();
    }
}
impl<'re> Renderable<'re> for &'re mut str {
    fn render(&self, cx: &mut Context) {
        writeln!(cx.output, "{}{self}", cx.indentation).unwrap();
    }
}
impl<'re> FlowContent<'re> for &'re str {}
impl<'re> PhrasingContent<'re> for &'re str {}
impl<'re> FlowContent<'re> for &'re mut str {}
impl<'re> PhrasingContent<'re> for &'re mut str {}

pub struct Body<'re> {
    pub children: Vec<'re, AnyElement<'re>>,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl<'re> Body<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Body {
            children: Vec::new_in(arena),
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        }
    }
}
derive_pre_render_hooks!('re, Body<'re>);
impl<'re> SimpleElement<'re> for Body<'re> {
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
    {
        GenericHtmlElement {
            name: "body",
            attributes: &[],
            children: strip_anyelement(arena, &self.children),
            late_children: &[],
        }
    }
}

#[derive(Clone)]
pub enum HtmlValue<'re> {
    Number(u32),
    String(&'re str),
    Bool(bool),
    Empty,
}
#[derive(Clone)]
pub struct HtmlAttribute<'re> {
    name: &'re str,
    value: HtmlValue<'re>,
}
impl<'re> Renderable<'re> for HtmlAttribute<'re> {
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
    pub children: &'re [&'re dyn Renderable<'re>],
    /// It will render after children, but displayed before.
    pub late_children: &'re [&'re dyn Renderable<'re>],
}
impl<'re> Renderable<'re> for GenericHtmlElement<'re> {
    fn render(&self, cx: &mut Context<'re>) {
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
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're;
}
fn strip_anyelement<'re, 'arena: 're>(
    arena: &'arena Bump,
    children: &Vec<'re, AnyElement<'re>>,
) -> &'re [&'re dyn Renderable<'re>] {
    let mut children_clone = Vec::new_in(arena);
    children_clone.extend(children.into_iter().map(|x| x.0));
    children_clone.into_bump_slice()
}
impl<'re, T> Renderable<'re> for T
where
    T: SimpleElement<'re, This = Self>,
{
    fn render(&self, cx: &mut Context<'re>) {
        if let Some(hook) = self.take_pre_render_hook() {
            hook(self, cx);
        }
        // SAFETY: hook called
        unsafe {
            self.into_html_element(cx.arena).render(cx);
        }
    }
}

type Hook<'re, This> = &'re mut dyn FnMut(&This, &mut Context);
pub trait PreRenderHooks<'re>: Sized
where
    Self: 're,
{
    type This;
    /// # Safety
    /// Hook should be unset(or taken out)
    unsafe fn set_pre_render_hook(&self, hook: impl FnMut(&Self::This, &mut Context) + 're);

    fn take_pre_render_hook(&self) -> Option<Hook<'re, Self::This>>;

    fn with_pre_render_hook(self, new_hook: impl FnMut(&Self::This, &mut Context) + 're) -> Self {
        self.add_pre_render_hook(new_hook);
        self
    }
    fn add_pre_render_hook(&self, mut new_hook: impl FnMut(&Self::This, &mut Context) + 're) {
        match self.take_pre_render_hook() {
            Some(prev_hook) => {
                // SAFETY: hook is taken out
                unsafe {
                    self.set_pre_render_hook(move |this, cx| {
                        prev_hook(this, cx);
                        new_hook(this, cx);
                    });
                }
            }
            None => {
                // SAFETY: hook is taken out
                unsafe {
                    self.set_pre_render_hook(new_hook);
                }
            }
        }
    }
}

struct PreRenderHookStorage<'re, This> {
    hook: Cell<Option<Hook<'re, This>>>,
    arena: &'re Bump,
}
impl<'re, T> PreRenderHookStorage<'re, T> {
    fn new_in(arena: &'re Bump) -> Self {
        PreRenderHookStorage {
            hook: Cell::new(None),
            arena,
        }
    }
}
impl<'re, This> PreRenderHooks<'re> for PreRenderHookStorage<'re, This> {
    type This = This;
    fn take_pre_render_hook(&self) -> Option<Hook<'re, This>> {
        self.hook.take()
    }
    unsafe fn set_pre_render_hook(&self, hook: impl FnMut(&This, &mut Context) + 're) {
        assert!(self.hook.take().is_none());
        let hook = self.arena.alloc(hook) as Hook<'re, This>;
        self.hook
            .set(Some(self.arena.alloc(|this: &This, cx: &mut Context| {
                hook(this, cx);
            })));
    }
}

pub struct Div<'re> {
    pub classes: Vec<'re, &'re str>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Div<'_> {
    fn class(mut self, class: &str) -> Self {
        assert!(!self.classes.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!class.is_empty());
        self.classes.push(self.arena.alloc_str(class));
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
impl<'re> FlowContent<'re> for Div<'re> {}
impl<'re> Div<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Div {
            classes: Vec::new_in(arena),
            id: None,
            children: Vec::new_in(arena),
            arena,
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        }
    }
    pub fn child(mut self, child: impl Component<'re> + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Div<'re> {
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
    {
        let mut attrs = Vec::new_in(arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if !self.classes.is_empty() {
            attrs.push(HtmlAttribute {
                name: "class",
                value: HtmlValue::String(arena.alloc_str(&self.classes.join(" "))),
            })
        }

        GenericHtmlElement {
            name: arena.alloc("div"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(arena, &self.children),
            late_children: &[],
        }
    }
}

pub struct Heading1<'re> {
    pub classes: Vec<'re, &'re str>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Heading1<'_> {
    fn class(mut self, class: &str) -> Self {
        assert!(!self.classes.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!class.is_empty());
        self.classes.push(self.arena.alloc_str(class));
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
impl<'re> FlowContent<'re> for Heading1<'re> {}
impl<'re> HeadingContent<'re> for Heading1<'re> {}
impl<'re> Heading1<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Heading1 {
            classes: Vec::new_in(arena),
            id: None,
            children: Vec::new_in(arena),
            arena,
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        }
    }
    pub fn child(mut self, child: impl PhrasingContent<'re> + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Heading1<'re> {
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
    {
        let mut attrs = Vec::new_in(arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if !self.classes.is_empty() {
            attrs.push(HtmlAttribute {
                name: "class",
                value: HtmlValue::String(arena.alloc_str(&self.classes.join(" "))),
            })
        }

        GenericHtmlElement {
            name: arena.alloc("h1"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(arena, &self.children),
            late_children: &[],
        }
    }
}

pub struct Paragraph<'re> {
    pub classes: Vec<'re, &'re str>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    arena: &'re Bump,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for Paragraph<'_> {
    fn class(mut self, class: &str) -> Self {
        assert!(!self.classes.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!class.is_empty());
        self.classes.push(self.arena.alloc_str(class));
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
impl<'re> FlowContent<'re> for Paragraph<'re> {}
impl<'re> Paragraph<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Paragraph {
            classes: Vec::new_in(arena),
            id: None,
            children: Vec::new_in(arena),
            arena,
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        }
    }
    pub fn child(mut self, child: impl PhrasingContent<'re> + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for Paragraph<'re> {
    unsafe fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
    {
        let mut attrs = Vec::new_in(arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if !self.classes.is_empty() {
            attrs.push(HtmlAttribute {
                name: "class",
                value: HtmlValue::String(arena.alloc_str(&self.classes.join(" "))),
            })
        }

        GenericHtmlElement {
            name: arena.alloc("p"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(arena, &self.children),
            late_children: &[],
        }
    }
}

pub fn html(arena: &Bump) -> Html<'_> {
    Html::new_in(arena)
}

pub fn div(arena: &Bump) -> Div<'_> {
    Div::new_in(arena)
}
pub fn h1(arena: &Bump) -> Heading1<'_> {
    Heading1::new_in(arena)
}
pub fn p(arena: &Bump) -> Paragraph<'_> {
    Paragraph::new_in(arena)
}
