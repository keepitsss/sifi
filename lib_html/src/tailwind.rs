use super::*;
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
}
impl<'re, T> TailwindExt<'re> for T where T: BuiltinHtmlElement + PreRenderHooks<'re> {}
