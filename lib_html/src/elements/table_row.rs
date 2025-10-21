use super::*;
pub struct TableRow<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, TableRow<'re>>,
}
impl BuiltinHtmlElement for TableRow<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, TableRow<'re>);

trait Sealed {}
#[allow(private_bounds)]
pub trait TdOrTh<'a>: IntoElement<'a> + Sealed {}
impl<'a> TdOrTh<'a> for TableDataCell<'a> {}
impl Sealed for TableDataCell<'_> {}
impl<'a> TdOrTh<'a> for TableHeaderCell<'a> {}
impl Sealed for TableHeaderCell<'_> {}

impl<'re> TableRow<'re> {
    pub fn child(mut self, child: impl TdOrTh<'re>) -> TableRow<'re> {
        self.children.push(child.into_any_element(self.arena));
        self
    }
    pub fn cells<DataOrHeaderCell: TdOrTh<'re>>(
        mut self,
        elements: impl IntoIterator<Item = DataOrHeaderCell>,
    ) -> TableRow<'re> {
        let mut count = 0;
        for element in elements {
            count += 1;
            self.children.push(element.into_any_element(self.arena));
        }
        assert!(count > 0, "You should provide at least one element");
        self
    }
}
impl<'re> SimpleElement<'re> for TableRow<'re> {
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
        GenericHtmlElement {
            name: self.arena.alloc("tr"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
