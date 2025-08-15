//! 're = 'rendering

use std::{cell::Cell, collections::HashSet, fmt::Write};

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

pub mod utils;

pub trait Renderable<'re> {
    fn render(&self, cx: &mut Context<'re>);
}

pub struct Context<'re> {
    pub indentation: utils::Indentation,
    pub output: std::string::String,
    pub arena: &'re Bump,
    pub ids: HashSet<&'re str>,
}

#[derive(Clone, Copy)]
pub struct AnyElement<'re>(pub &'re dyn Renderable<'re>);
impl<'re> Renderable<'re> for AnyElement<'re> {
    fn render(&self, cx: &mut Context<'re>) {
        self.0.render(cx);
    }
}
pub trait Component<'re>: Renderable<'re> {
    fn into_any_element<'arena>(self, arena: &'arena Bump) -> AnyElement<'re>
    where
        'arena: 're,
        Self: 're + Sized,
    {
        let value = arena.alloc(self);
        AnyElement(value)
    }
}

pub struct Html<'re> {
    pub head: Head<'re>,
    pub body: Body<'re>,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl<'re> PreRenderHooks<'re> for Html<'re> {
    type This = Self;
    unsafe fn set_pre_render_hook(&self, hook: impl FnMut(&Self, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn take_pre_render_hook(&self) -> Option<Hook<'re, Self>> {
        self.pre_render_hook.take_pre_render_hook()
    }
}
impl<'re> Html<'re> {
    pub fn add_to_body(&mut self, body: impl Component<'re> + 're) {
        self.body
            .children
            .push(body.into_any_element(self.body.children.bump()));
    }
}
impl<'re> SimpleElement<'re> for Html<'re> {
    fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
    {
        GenericHtmlElement {
            name: "html",
            attributes: &[],
            children: bumpalo::vec![in arena; arena.alloc(self.body.into_html_element(arena)) as &dyn Renderable]
                .into_bump_slice(),
            late_children: bumpalo::vec![in arena; arena.alloc(self.head.into_html_element(arena)) as &dyn Renderable]
                            .into_bump_slice()
        }
    }
}
pub struct Head<'re> {
    pub styles: Vec<'re, &'re str>,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl<'re> Head<'re> {
    pub fn add_style<'arena: 're>(&mut self, style: &str) {
        self.styles.push(self.styles.bump().alloc_str(style.trim()));
    }
}
impl<'re> PreRenderHooks<'re> for Head<'re> {
    type This = Self;
    unsafe fn set_pre_render_hook(&self, hook: impl FnMut(&Self, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn take_pre_render_hook(&self) -> Option<Hook<'re, Self>> {
        self.pre_render_hook.take_pre_render_hook()
    }
}
impl<'re> SimpleElement<'re> for Head<'re> {
    fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
    {
        let mut children = Vec::new_in(arena);
        if !self.styles.is_empty() {
            children.push(arena.alloc(Style(self.styles.clone())) as &dyn Renderable);
        }
        GenericHtmlElement {
            name: "head",
            attributes: &[],
            children: children.into_bump_slice(),
            late_children: &[],
        }
    }
}
#[derive(Clone)]
struct Style<'re>(Vec<'re, &'re str>);
impl<'re> Renderable<'re> for Style<'re> {
    fn render(&self, cx: &mut Context) {
        assert!(!self.0.is_empty());
        cx_writeln!(cx, "{}<style>", cx.indentation);
        cx.indentation.level += 1;
        cx_writeln!(cx, "");
        for i in 0..self.0.len() {
            cx_writeln!(cx, "{}{}", /* cx.indentation */ "", self.0[i]);
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
impl<'re> Component<'re> for &'re str {}
impl<'re> Component<'re> for &'re mut str {}

pub struct Body<'re> {
    pub children: Vec<'re, AnyElement<'re>>,
    pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl<'re> PreRenderHooks<'re> for Body<'re> {
    type This = Self;
    unsafe fn set_pre_render_hook(&self, hook: impl FnMut(&Self, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn take_pre_render_hook(&self) -> Option<Hook<'re, Self>> {
        self.pre_render_hook.take_pre_render_hook()
    }
}
impl<'re> SimpleElement<'re> for Body<'re> {
    fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
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

pub struct HtmlElementLazy<T> {
    pub inner: T,
    pub pre_render: PreRenderFn<T>,
}

type PreRenderFn<T> = fn(&T, &mut Context) -> Result<(), std::string::String>;
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

pub trait SimpleElement<'re>: PreRenderHooks<'re, This = Self> {
    #[allow(clippy::wrong_self_convention)]
    fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
        Self: 're;
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
    T: SimpleElement<'re>,
{
    fn render(&self, cx: &mut Context<'re>) {
        if let Some(hook) = self.take_pre_render_hook() {
            hook(self, cx);
        }
        self.into_html_element(cx.arena).render(cx);
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

    fn with_pre_render_hook(
        self,
        mut new_hook: impl FnMut(&Self::This, &mut Context) + 're,
    ) -> Self {
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
        self
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
impl<'re> PreRenderHooks<'re> for Div<'re> {
    type This = Self;
    unsafe fn set_pre_render_hook(&self, hook: impl FnMut(&Self, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn take_pre_render_hook(&self) -> Option<Hook<'re, Self>> {
        self.pre_render_hook.take_pre_render_hook()
    }
}
impl<'re> Component<'re> for Div<'re> {}
impl<'re> Div<'re> {
    pub fn child(mut self, child: impl Component<'re> + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
    pub fn id(mut self, id: &str) -> Self {
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
    pub fn class(mut self, class: &str) -> Self {
        assert!(!self.classes.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!class.is_empty());
        self.classes.push(self.arena.alloc_str(class));
        self
    }
    pub fn classes<'a>(mut self, classes: impl IntoIterator<Item = &'a str>) -> Self {
        let mut count = 0;
        for class in classes.into_iter() {
            self = self.class(class);
            count += 1;
        }
        assert_ne!(count, 0, "empty classes provided");
        assert_ne!(count, 1, "use 'class' method to provide one class");
        self
    }
}
impl<'re> SimpleElement<'re> for Div<'re> {
    fn into_html_element<'arena>(&self, arena: &'arena Bump) -> GenericHtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
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

pub fn html<'re, 'arena: 're>(arena: &'arena Bump) -> Html<'re> {
    Html {
        head: Head {
            styles: Vec::new_in(arena),
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        },
        body: Body {
            children: Vec::new_in(arena),
            pre_render_hook: PreRenderHookStorage::new_in(arena),
        },
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}

pub fn div<'re, 'arena: 're>(arena: &'arena Bump) -> Div<'re> {
    Div {
        classes: Vec::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
