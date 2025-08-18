use super::*;

// FIXME: make it order-independent

pub trait TailwindExt<'re>: BuiltinHtmlElement + PreRenderHooks<'re> {
    fn font_sans(self) -> Self {
        self.class("font-sans").with_pre_render_hook(|_this, cx| {
            cx.styles.insert(
                "
.font-sans {
    font-family: sans-serif;
}
                ",
            );
        })
    }
    fn margin(self, size: f32) -> Self {
        self.class(&format!("m-{size}"))
            .with_pre_render_hook(move |_this, cx| {
                cx.styles.insert(cx.arena.alloc_str(&format!(
                    "
.m-{size} {{
    margin: {}rem;
}}
                    ",
                    size / 4.
                )));
            })
    }
    fn margin_x(self, size: f32) -> Self {
        self.class(&format!("mx-{size}"))
            .with_pre_render_hook(move |_this, cx| {
                cx.styles.insert(cx.arena.alloc_str(&format!(
                    "
.mx-{size} {{
    margin-left: {0}rem;
    margin-right: {0}rem;
}}
                    ",
                    size / 4.
                )));
            })
    }
    fn margin_x_auto(self) -> Self {
        self.class("mx-auto")
            .with_pre_render_hook(move |_this, cx| {
                cx.styles.insert(
                    "
.mx-auto {
    margin-left: auto;
    margin-right: auto;
}
                    ",
                );
            })
    }
}
impl<'re, T> TailwindExt<'re> for T where T: BuiltinHtmlElement + PreRenderHooks<'re> {}
