use super::*;
#[derive(Clone)]
pub struct Html<'re> {
    pub(crate) head: Head<'re>,
    pub(crate) body: Option<Body<'re>>,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
    pub(crate) arena: &'re Bump,
}
derive_pre_render_hooks!('re, Html<'re>);
impl<'re> Html<'re> {
    pub fn body(&mut self, body: Body<'re>) {
        assert!(self.body.is_none());
        self.body = Some(body);
    }
}
impl<'re> SimpleElement<'re> for Html<'re> {
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        GenericHtmlElement {
            name: "html",
            attributes: &[],
            children:
                bumpalo::vec![in self.arena; self.arena.alloc(self.body.clone().expect("body should be set")) as &dyn Renderable]
                    .into_bump_slice(),
                late_children: bumpalo::vec![in self.arena; self.arena.alloc(self.head.clone()) as &dyn Renderable].into_bump_slice(),
        }
    }
}
