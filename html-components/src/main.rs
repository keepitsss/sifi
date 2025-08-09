//! 're = 'rendering

use std::fmt::{Display, Write};

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

struct Indentation {
    level: u32,
    width: u8,
}
impl Default for Indentation {
    fn default() -> Self {
        Self { level: 0, width: 2 }
    }
}
impl Display for Indentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            " ".repeat((self.level * self.width as u32) as usize)
        )
    }
}

#[derive(Clone, Copy)]
struct AnyElement<'rendering>(&'rendering dyn Renderable);
impl Renderable for AnyElement<'_> {
    fn render(&self, cx: &mut Context) {
        self.0.render(cx);
    }
}
trait IntoElement {
    fn into_any_element<'rendering, 'arena>(self, arena: &'arena Bump) -> AnyElement<'rendering>
    where
        'arena: 'rendering,
        Self: 'rendering;
}
impl<T> IntoElement for T
where
    T: Renderable,
{
    fn into_any_element<'rendering, 'arena>(self, arena: &'arena Bump) -> AnyElement<'rendering>
    where
        'arena: 'rendering,
        T: 'rendering,
    {
        let value = arena.alloc(self);
        AnyElement(value)
    }
}

struct Context<'rendering> {
    indentation: Indentation,
    output: String,
    arena: &'rendering Bump,
}
trait Renderable {
    fn render(&self, cx: &mut Context);
}

struct Html<'rendering> {
    body: Option<AnyElement<'rendering>>,
}

impl Renderable for Html<'_> {
    fn render(&self, cx: &mut Context) {
        cx_writeln!(cx, "{}<html>", cx.indentation);
        cx.indentation.level += 1;
        cx_writeln!(cx, "{}<body>", cx.indentation);
        if let Some(body) = &self.body {
            cx.indentation.level += 1;
            body.render(cx);
            cx.indentation.level -= 1;
        }
        cx_writeln!(cx, "{}</body>", cx.indentation);
        cx.indentation.level -= 1;
        cx_writeln!(cx, "{}</html>", cx.indentation);
    }
}

#[derive(Clone)]
enum HtmlValue {
    Number(u32),
    String(String),
    Bool(bool),
    Empty,
}
#[derive(Clone)]
struct HtmlAttribute {
    name: String,
    value: HtmlValue,
}
impl Renderable for HtmlAttribute {
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
struct HtmlElement<'rendering> {
    name: &'rendering str,
    attributes: &'rendering [HtmlAttribute],
    children: &'rendering [AnyElement<'rendering>],
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

trait SimpleElement {
    #[allow(clippy::wrong_self_convention)]
    fn into_html_element<'rendering, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'rendering>
    where
        Self: 'rendering;
}
impl<T> Renderable for T
where
    T: SimpleElement,
{
    fn render(&self, cx: &mut Context) {
        self.into_html_element(cx.arena).render(cx);
    }
}

struct Div<'a> {
    children: Vec<'a, AnyElement<'a>>,
}

impl SimpleElement for Div<'_> {
    fn into_html_element<'rendering, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'rendering>
    where
        Self: 'rendering,
    {
        HtmlElement {
            name: arena.alloc("div"),
            attributes: &[],
            children: self.children.clone().into_bump_slice(),
        }
    }
}

fn main() {
    let allocator = Bump::new();

    let mut cx = Context {
        indentation: Indentation::default(),
        output: String::new(),
        arena: &allocator,
    };
    let arena = cx.arena;

    let mut html = html();
    let div2 = div(arena).with_child(arena, div(arena));
    html.set_body(arena, div2);

    html.render(&mut cx);

    let output = cx.output;
    drop(allocator);

    println!("{output}");
}

impl<'re> Div<'re> {
    fn add_child(&mut self, div1: impl IntoElement + 're) {
        self.children
            .push(div1.into_any_element(self.children.bump()));
    }
    fn with_child<'arena>(mut self, arena: &'arena Bump, child: impl IntoElement + 're) -> Self
    where
        'arena: 're,
    {
        self.add_child(child.into_any_element(arena));
        self
    }
}

impl<'re> Html<'re> {
    fn set_body<'arena>(&mut self, arena: &'arena Bump, body: impl IntoElement + 're)
    where
        'arena: 're,
    {
        self.body = Some(body.into_any_element(arena));
    }
}

fn html<'re>() -> Html<'re> {
    Html { body: None }
}

fn div<'re, 'arena>(arena: &'arena Bump) -> Div<'re>
where
    'arena: 're,
{
    Div {
        children: Vec::new_in(arena),
    }
}
