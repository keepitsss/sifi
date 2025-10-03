
use super::*;

#[derive(Clone, Copy)]
pub struct OrderedListMarkerType(&'static str);
impl OrderedListMarkerType {
    /// Decimal numbers
    pub const DECIMAL: Self = Self("1");
    /// Lowercase latin alphabet
    pub const LOWER_ALPHA: Self = Self("a");
    /// Uppercase latin alphabet
    pub const UPPER_ALPHA: Self = Self("A");
    /// Lowercase roman numerals
    pub const LOWER_ROMAN: Self = Self("i");
    /// Uppercase roman numerals
    pub const UPPER_ROMAN: Self = Self("I");
}

pub struct OrderedList<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub reversed: bool,
    pub starting_value: Option<i32>,
    pub marker_type: Option<OrderedListMarkerType>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}
impl BuiltinHtmlElement for OrderedList<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, OrderedList<'re>);
impl FlowContent for OrderedList<'_> {}
impl<'re> OrderedList<'re> {
    #[allow(private_bounds)]
    pub fn child<NoConstraints: ListItemValueProp>(
        mut self,
        child: ListItem<'re, NoConstraints>,
    ) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
    /// If present, it indicates that the list is a descending list (..., 3, 2, 1).
    /// If the attribute is omitted, the list is an ascending list (1, 2, 3, ...).
    pub fn reversed(mut self) -> Self {
        assert!(!self.reversed);
        self.reversed = true;
        self
    }
    /// It is used to determine the starting value of the list.
    pub fn start(mut self, starting_value: i32) -> Self {
        assert!(self.starting_value.is_none());
        self.starting_value = Some(starting_value);
        self
    }
    pub fn marker_type(mut self, marker_type: OrderedListMarkerType) -> Self {
        assert!(self.marker_type.is_none());
        self.marker_type = Some(marker_type);
        self
    }
}
impl<'re> SimpleElement<'re> for OrderedList<'re> {
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
        if self.reversed {
            attrs.push(HtmlAttribute {
                name: "reversed",
                value: HtmlValue::Bool,
            });
        }
        if let Some(start) = self.starting_value {
            attrs.push(HtmlAttribute {
                name: "start",
                value: HtmlValue::Number(start),
            });
        }
        if let Some(marker_type) = self.marker_type {
            attrs.push(HtmlAttribute {
                name: "type",
                value: HtmlValue::String(marker_type.0),
            })
        }
        GenericHtmlElement {
            name: self.arena.alloc("ol"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
