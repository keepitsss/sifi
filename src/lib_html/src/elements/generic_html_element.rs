use super::*;
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

pub trait SimpleElement<'re>: PreRenderHooks<'re, This = Self::GenericSelf> {
    type GenericSelf; // = Self;
    #[allow(clippy::wrong_self_convention)]
    /// # Safety
    /// You should call pre_render_hooks before rendering
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re>;
}
pub fn strip_anyelement<'re>(
    arena: &'re Bump,
    children: &Vec<'re, AnyElement<'re>>,
) -> &'re [&'re dyn Renderable] {
    let mut children_clone = Vec::new_in(arena);
    children_clone.extend(children.into_iter().map(|x| x.0));
    children_clone.into_bump_slice()
}
impl<'re, T> Renderable for T
where
    T: SimpleElement<'re>,
    for<'a> &'a T::GenericSelf: From<&'a T>,
{
    fn render(&self, cx: &mut Context) {
        if let Some(hook) = self.get_pre_render_hook() {
            hook(self.into(), cx);
        }
        // SAFETY: hook called
        unsafe {
            self.into_html_element().render(cx);
        }
    }
}

#[derive(Clone, Copy)]
pub enum HtmlValue<'re> {
    Number(i32),
    String(&'re str),
    Bool,
}
#[derive(Clone, Copy)]
pub struct HtmlAttribute<'re> {
    pub name: &'re str,
    pub value: HtmlValue<'re>,
}
impl<'re> Renderable for HtmlAttribute<'re> {
    fn render(&self, cx: &mut Context) {
        let name = &self.name;
        match &self.value {
            HtmlValue::Number(number) => cx_write!(cx, " {name}=\"{number}\""),
            HtmlValue::String(string) => cx_write!(cx, " {name}=\"{string}\""), // FIXME: escaping
            HtmlValue::Bool => cx_write!(cx, " {name}"),
        }
    }
}
