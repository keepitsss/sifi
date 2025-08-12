//! 're = 'rendering

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

pub mod utils;

pub trait Renderable {
    fn render(&self, cx: &mut Context);
}

pub struct Context<'re> {
    pub indentation: utils::Indentation,
    pub output: String,
    pub arena: &'re Bump,
    pub ids: HashSet<String>,
}

#[derive(Clone, Copy)]
pub struct AnyElement<'re>(pub &'re dyn Renderable);

pub trait IntoElement {
    fn into_any_element<'re, 'arena>(self, arena: &'arena Bump) -> AnyElement<'re>
    where
        'arena: 're,
        Self: 're;
}

//

impl Renderable for AnyElement<'_> {
    fn render(&self, cx: &mut Context) {
        self.0.render(cx);
    }
}
impl<T> IntoElement for T
where
    T: Renderable,
{
    fn into_any_element<'re, 'arena>(self, arena: &'arena Bump) -> AnyElement<'re>
    where
        'arena: 're,
        T: 're,
    {
        let value = arena.alloc(self);
        AnyElement(value)
    }
}

pub struct Html<'re> {
    pub head: Head<'re>,
    pub body: Body<'re>,
}
impl<'re> Html<'re> {
    pub fn add_to_body<'arena: 're>(&mut self, body: impl IntoElement + 're) {
        self.body
            .children
            .push(body.into_any_element(self.body.children.bump()));
    }
}
impl SimpleElement for Html<'_> {
    fn into_html_element<'re, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
    {
        HtmlElement {
            name: "html",
            attributes: &[],
            children: bumpalo::vec![in arena; self.head.clone().into_any_element(arena), self.body.clone().into_any_element(arena)]
                .into_bump_slice(),
        }
    }
}
#[derive(Clone)]
pub struct Head<'re> {
    pub children: Vec<'re, AnyElement<'re>>,
}
impl SimpleElement for Head<'_> {
    fn into_html_element<'re, 'arena>(&self, _arena: &'arena Bump) -> HtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
    {
        HtmlElement {
            name: "head",
            attributes: &[],
            children: self.children.clone().into_bump_slice(),
        }
    }
}
#[derive(Clone)]
pub struct Body<'re> {
    pub children: Vec<'re, AnyElement<'re>>,
}
impl SimpleElement for Body<'_> {
    fn into_html_element<'re, 'arena>(&self, _arena: &'arena Bump) -> HtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
    {
        HtmlElement {
            name: "body",
            attributes: &[],
            children: self.children.clone().into_bump_slice(),
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
impl Renderable for HtmlAttribute<'_> {
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

type PreRenderFn<T> = fn(&T, &mut Context) -> Result<(), String>;
#[derive(Clone)]
pub struct HtmlElement<'re> {
    pub name: &'re str,
    pub attributes: &'re [HtmlAttribute<'re>],
    pub children: &'re [AnyElement<'re>],
}
impl Renderable for HtmlElement<'_> {
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

        for child in self.children {
            cx.indentation.level += 1;
            child.render(&mut *cx);
            cx.indentation.level -= 1;
        }

        cx_writeln!(cx, "{}</{}>", cx.indentation, self.name);
    }
}

pub trait SimpleElement {
    #[allow(clippy::wrong_self_convention)]
    fn into_html_element<'re, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'re>
    where
        'arena: 're,
        Self: 're;

    fn pre_render_checks(&self, _cx: &mut Context) -> Result<(), String> {
        Ok(())
    }
}
impl<T> Renderable for T
where
    T: SimpleElement,
{
    fn render(&self, cx: &mut Context) {
        self.pre_render_checks(cx).unwrap(); // FIXME
        self.into_html_element(cx.arena).render(cx);
    }
}

pub struct Div<'re> {
    pub classes: Vec<'re, &'re str>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    arena: &'re Bump,
}
impl<'re> Div<'re> {
    pub fn child(mut self, child: impl IntoElement + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
    pub fn id(mut self, id: &str) -> Self {
        assert!(self.id.is_none());
        assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
        self.id = Some(self.arena.alloc_str(id));
        self
    }
    pub fn class(mut self, class: &str) -> Self {
        assert!(!self.classes.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        self.classes.push(self.arena.alloc_str(class));
        self
    }
}
impl SimpleElement for Div<'_> {
    fn into_html_element<'re, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'re>
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

        HtmlElement {
            name: arena.alloc("div"),
            attributes: attrs.into_bump_slice(),
            children: self.children.clone().into_bump_slice(),
        }
    }

    fn pre_render_checks(&self, cx: &mut Context) -> Result<(), String> {
        if let Some(id) = self.id {
            if cx.ids.contains(id) {
                return Err(format!("'{id}' id duplicate"));
            }
            cx.ids.insert(id.into());
        }
        Ok(())
    }
}

pub fn html<'re, 'arena: 're>(arena: &'arena Bump) -> Html<'re> {
    Html {
        head: Head {
            children: Vec::new_in(arena),
        },
        body: Body {
            children: Vec::new_in(arena),
        },
    }
}

pub fn div<'re, 'arena: 're>(arena: &'arena Bump) -> Div<'re> {
    Div {
        classes: Vec::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
    }
}
