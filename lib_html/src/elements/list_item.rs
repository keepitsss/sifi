use std::any::Any;

use super::*;
pub struct NoValue;
impl ListItemValueProp for NoValue {}
pub struct WithValue(pub i32);
impl ListItemValueProp for WithValue {}
#[diagnostic::on_unimplemented(message = "Use `NoValue` or `WithValue` struct instead")]
pub(crate) trait ListItemValueProp: Any {}
pub struct ListItem<'re, IsOrdered> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub value: IsOrdered,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, ListItem<'re, IsOrdered>>,
}
impl BuiltinHtmlElement for ListItem<'_, WithChild> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re + IsOrdered, ListItem<'re, IsOrdered>);
impl<'re, IsOrdered> ListItem<'re, IsOrdered> {
    pub fn child(mut self, child: impl FlowContent + 're) -> ListItem<'re, IsOrdered> {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re, IsOrdered: ListItemValueProp> SimpleElement<'re> for ListItem<'re, IsOrdered> {
    type GenericSelf = Self;
    unsafe fn into_html_element(&self) -> GenericHtmlElement<'re> {
        let mut attrs = Vec::new_in(self.arena);
        if let Some(id) = self.id {
            attrs.push(HtmlAttribute {
                name: "id",
                value: HtmlValue::String(id),
            });
        }
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        if let Some(WithValue(value)) = (&self.value as &dyn Any).downcast_ref() {
            attrs.push(HtmlAttribute {
                name: "value",
                value: HtmlValue::Number(*value),
            });
        }
        GenericHtmlElement {
            name: self.arena.alloc("li"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
