use std::{
    fmt::{Display, Write},
    rc::Rc,
};

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

struct Context {
    indentation: Indentation,
    output: String,
}
trait Renderable {
    fn render(&self, cx: &mut Context);
}
impl Context {
    fn child(&mut self, child: impl Into<Rc<dyn Renderable>>) -> &mut Self {
        let child = child.into();
        self.indentation.level += 1;
        child.render(self);
        self.indentation.level -= 1;
        self
    }
    fn child_primitive(&mut self, rendering: impl FnOnce(&mut Self)) -> &mut Self {
        self.indentation.level += 1;
        rendering(self);
        self.indentation.level -= 1;
        self
    }
    fn root(&mut self, child: impl Renderable) -> &mut Self {
        child.render(self);
        self
    }
}

#[derive(Default)]
struct Html {
    body: Option<Rc<dyn Renderable>>,
}
fn html() -> Html {
    Html::default()
}
impl Html {
    fn body(mut self, child: Div) -> Self {
        assert!(self.body.is_none());
        self.body = Some(Rc::new(child));
        self
    }
}
impl Renderable for Html {
    fn render(&self, cx: &mut Context) {
        cx_writeln!(cx, "{}<html>", cx.indentation);
        cx.child_primitive(|cx| {
            cx_writeln!(cx, "{}<body>", cx.indentation);
            if let Some(body) = &self.body {
                cx.child(body.clone());
            }
            cx_writeln!(cx, "{}</body>", cx.indentation);
        });
        cx_writeln!(cx, "{}</html>", cx.indentation);
    }
}

enum HtmlValue {
    Number(u32),
    String(String),
    Bool(bool),
    Empty,
}
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
struct HtmlElement {
    name: String,
    attributes: Vec<HtmlAttribute>,
    children: Vec<Rc<dyn Renderable>>,
}
impl Renderable for HtmlElement {
    fn render(&self, cx: &mut Context) {
        {
            cx_write!(cx, "{}<{}", cx.indentation, self.name);
            for attribute in &self.attributes {
                attribute.render(cx);
            }
            if self.children.is_empty() {
                cx_writeln!(cx, "/>");
                return;
            }
            cx_writeln!(cx, ">");
        }

        for child in &self.children {
            cx.child(child.clone());
        }

        cx_writeln!(cx, "{}</{}>", cx.indentation, self.name);
    }
}
impl HtmlElement {
    fn new(name: String) -> Self {
        Self {
            name,
            attributes: vec![],
            children: vec![],
        }
    }
    fn child(&mut self, child: Rc<dyn Renderable>) {
        self.children.push(child);
    }
}

struct Div {
    inner: HtmlElement,
}
fn div() -> Div {
    Div {
        inner: HtmlElement::new("div".into()),
    }
}
impl Div {
    fn child(mut self, child: impl Renderable + 'static) -> Self {
        self.inner.child(Rc::new(child));
        self
    }
}
impl Renderable for Div {
    fn render(&self, cx: &mut Context) {
        self.inner.render(cx)
    }
}

fn main() {
    let html = html().body(div().child(div()).child(div()));

    let mut cx = Context {
        indentation: Indentation::default(),
        output: String::new(),
    };
    html.render(&mut cx);
    println!("{}", cx.output);
}
