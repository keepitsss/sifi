use std::{
    any::Any,
    fmt::{Display, Write},
};

use bumpalo::Bump;

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
struct AnyElement<'a>(&'a dyn Renderable);
impl Renderable for AnyElement<'_> {
    fn render(&self, cx: &mut Context) {
        self.0.render(cx);
    }
}
trait IntoElement {
    fn into_any_element<'a, 'arena>(self, arena: &'arena Bump) -> AnyElement<'a>
    where
        'arena: 'a;
}
impl<T> IntoElement for T
where
    T: Renderable + 'static,
{
    fn into_any_element<'a, 'arena>(self, arena: &'arena Bump) -> AnyElement<'a>
    where
        'arena: 'a,
    {
        let value = arena.alloc(self);
        AnyElement(value)
    }
}

struct Context<'a> {
    indentation: Indentation,
    output: String,
    arena: &'a Bump,
}
trait Renderable {
    fn render(&self, cx: &mut Context);
}

struct Html<'a> {
    body: Option<AnyElement<'a>>,
}

impl<'a> Html<'a> {
    fn body<'arena>(&mut self, arena: &'arena Bump, child: impl IntoElement) -> &mut Self
    where
        'arena: 'a,
    {
        assert!(self.body.is_none());
        self.body = Some(child.into_any_element(arena));
        self
    }
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
struct HtmlElement<'a> {
    name: &'a str,
    attributes: &'a [HtmlAttribute],
    children: &'a [AnyElement<'a>],
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
    fn into_html_element<'a, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'a>;
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
    children: Vec<AnyElement<'a>>,
}

impl<'a> Div<'a> {
    fn child<'arena>(mut self, arena: &'arena Bump, child: impl Renderable + 'static) -> Self
    where
        'arena: 'a,
    {
        self.children.push(child.into_any_element(arena));
        self
    }
}
impl<'elem> SimpleElement for Div<'elem> {
    fn into_html_element<'a, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'a> {
        // TODO: copy vec of children to arena and pass reference
        HtmlElement {
            name: arena.alloc("div"),
            attributes: &[],
            children: &[], // FIXME
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
    let mut html = html();
    let body = div().child(cx.arena, div());
    html.set_body(cx.arena, body);

    html.render(&mut cx);

    let output = cx.output;
    drop(allocator);

    println!("{output}");
}

impl<'a> Html<'a> {
    fn set_body<'arena>(&mut self, arena: &'arena Bump, body: Div<'a>)
    where
        'arena: 'a,
    {
        self.body = Some(AnyElement(arena.alloc(body)));
    }
}

fn html<'a>() -> Html<'a> {
    Html { body: None }
}

fn div<'a>() -> Div<'a> {
    Div {
        children: Vec::new(),
    }
}
