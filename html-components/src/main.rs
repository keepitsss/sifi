//! 're = 'rendering

use std::fmt::Write;

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

mod utils;

trait Renderable {
    fn render(&self, cx: &mut Context);
}

struct Context<'re> {
    indentation: utils::Indentation,
    output: String,
    arena: &'re Bump,
}

#[derive(Clone, Copy)]
struct AnyElement<'re>(&'re dyn Renderable);

trait IntoElement {
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

struct Html<'re> {
    body: Body<'re>,
}
impl<'re> Html<'re> {
    fn set_body<'arena: 're>(&mut self, arena: &'arena Bump, body: impl IntoElement + 're) {
        self.body.children.push(body.into_any_element(arena));
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
            children: bumpalo::vec![in arena; self.body.clone().into_any_element(arena)]
                .into_bump_slice(),
        }
    }
}
#[derive(Clone)]
struct Body<'re> {
    children: Vec<'re, AnyElement<'re>>,
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
enum HtmlValue<'re> {
    Number(u32),
    String(&'re str),
    Bool(bool),
    Empty,
}
#[derive(Clone)]
struct HtmlAttribute<'re> {
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
#[derive(Clone)]
struct HtmlElement<'re> {
    name: &'re str,
    attributes: &'re [HtmlAttribute<'re>],
    children: &'re [AnyElement<'re>],
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
    fn into_html_element<'re, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'re>
    where
        'arena: 're,
        Self: 're;
}
impl<T> Renderable for T
where
    T: SimpleElement,
{
    fn render(&self, cx: &mut Context) {
        self.into_html_element(cx.arena).render(cx);
    }
}

struct Div<'re> {
    children: Vec<'re, AnyElement<'re>>,
}
impl<'re> Div<'re> {
    fn add_child(&mut self, div1: impl IntoElement + 're) {
        self.children
            .push(div1.into_any_element(self.children.bump()));
    }
    fn with_child<'arena: 're>(
        mut self,
        arena: &'arena Bump,
        child: impl IntoElement + 're,
    ) -> Self {
        self.add_child(child.into_any_element(arena));
        self
    }
}
impl SimpleElement for Div<'_> {
    fn into_html_element<'re, 'arena>(&self, arena: &'arena Bump) -> HtmlElement<'re>
    where
        'arena: 're,
        Self: 're,
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
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: &allocator,
    };
    let arena = cx.arena;

    let mut html = html(arena);
    let div2 = div(arena).with_child(arena, div(arena));
    html.set_body(arena, div2);

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    drop(allocator);

    println!("{output}");
}

fn html<'re, 'arena: 're>(arena: &'arena Bump) -> Html<'re> {
    Html {
        body: Body {
            children: Vec::new_in(arena),
        },
    }
}

fn div<'re, 'arena: 're>(arena: &'arena Bump) -> Div<'re> {
    Div {
        children: Vec::new_in(arena),
    }
}
