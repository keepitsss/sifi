use std::fmt::{Display, Write};

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
}
trait Renderable {
    fn render(self: Box<Self>, output: &mut String, cx: &mut Context) -> std::fmt::Result;
}

#[derive(Default)]
struct Html {
    body: Option<Box<Div>>,
}
fn html() -> Html {
    Html::default()
}
impl Html {
    fn body(mut self, child: Box<Div>) -> Self {
        assert!(self.body.is_none());
        self.body = Some(child);
        self
    }
}
impl Renderable for Html {
    fn render(self: Box<Self>, output: &mut String, cx: &mut Context) -> std::fmt::Result {
        writeln!(output, "{}<html>", cx.indentation)?;
        cx.indentation.level += 1;
        writeln!(output, "{}<body>", cx.indentation)?;
        if let Some(body) = self.body {
            cx.indentation.level += 1;
            body.render(output, cx)?;
            cx.indentation.level -= 1;
        }
        writeln!(output, "{}</body>", cx.indentation)?;
        cx.indentation.level -= 1;
        writeln!(output, "{}</html>", cx.indentation)?;
        Ok(())
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
    fn render(self: Box<Self>, output: &mut String, cx: &mut Context) -> std::fmt::Result {
        let name = self.name;
        match self.value {
            HtmlValue::Number(number) => write!(output, " {name}={number}"),
            HtmlValue::String(string) => write!(output, " {name}=\"{string}\""), // FIXME: escaping
            HtmlValue::Bool(bool) => write!(output, " {name}={bool}"),
            HtmlValue::Empty => write!(output, " {name}"),
        }
    }
}
struct HtmlElement {
    name: String,
    attributes: Vec<HtmlAttribute>,
    children: Vec<Box<dyn Renderable>>,
}
impl Renderable for HtmlElement {
    fn render(self: Box<Self>, output: &mut String, cx: &mut Context) -> std::fmt::Result {
        {
            write!(output, "{}<{}", cx.indentation, self.name)?;
            for attribute in self.attributes {
                Box::new(attribute).render(output, cx)?;
            }
            if self.children.is_empty() {
                writeln!(output, "/>")?;
                return Ok(());
            }
            writeln!(output, ">")?;
        }

        cx.indentation.level += 1;
        for child in self.children {
            child.render(output, cx)?;
        }
        cx.indentation.level -= 1;

        writeln!(output, "{}</{}>", cx.indentation, self.name)?;
        Ok(())
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
    fn child(&mut self, child: Box<dyn Renderable>) {
        self.children.push(child);
    }
}

struct Div {
    inner: HtmlElement,
}
fn div() -> Box<Div> {
    Box::new(Div {
        inner: HtmlElement::new("div".into()),
    })
}
impl Div {
    fn child(mut self: Box<Self>, child: Box<dyn Renderable>) -> Box<Self> {
        self.inner.child(child);
        self
    }
}
impl Renderable for Div {
    fn render(self: Box<Self>, output: &mut String, cx: &mut Context) -> std::fmt::Result {
        Box::new(self.inner).render(output, cx)
    }
}

fn main() {
    let html = html().body(div().child(div()).child(div()));

    let mut cx = Context {
        indentation: Indentation::default(),
    };
    let mut output = String::new();
    Renderable::render(Box::new(html), &mut output, &mut cx).unwrap();
    println!("{output}");
}
