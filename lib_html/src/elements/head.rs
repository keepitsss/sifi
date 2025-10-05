use super::*;
#[derive(Clone)]
pub struct Head<'re> {
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
    pub(crate) arena: &'re Bump,
}
derive_pre_render_hooks!('re, Head<'re>);
impl<'re> SimpleElement<'re> for Head<'re> {
    type GenericSelf = Self;
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
