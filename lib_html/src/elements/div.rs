use super::*;

pub trait DivType {}
pub struct DescriptionListDiv;
impl DivType for DescriptionListDiv {}
pub struct OptionDiv;
impl DivType for OptionDiv {}
pub struct OptionGroupDiv;
impl DivType for OptionGroupDiv {}
pub struct SelectDiv;
impl DivType for SelectDiv {}
pub struct GeneralDiv;
impl DivType for GeneralDiv {}

pub struct Div<'re, Type: DivType, HasChild> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Div<'re, Type, WithChild>>,
    pub(crate) marker: PhantomData<Type>,
    pub(crate) has_child: PhantomData<HasChild>,
}
impl<'re, Type: DivType, HasChild: 're> BuiltinHtmlElement for Div<'re, Type, HasChild> {
    derive_class!();
    derive_id!();
}
impl<'re, Type: DivType, HasChild: 're> PreRenderHooks<'re> for Div<'re, Type, HasChild> {
    type This = Div<'re, Type, WithChild>;
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>> {
        self.pre_render_hook.get_pre_render_hook()
    }
}
impl FlowContent for Div<'_, GeneralDiv, WithChild> {}
impl<'re, HasChild> Div<'re, GeneralDiv, HasChild> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Div<'re, GeneralDiv, WithChild> {
        self.children.push(child.into_any_element(self.arena));
        let Div {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            marker,
            has_child: _,
        } = self;
        Div {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            marker,
            has_child: PhantomData,
        }
    }
}
// FIXME: add child method for other modes

impl<'re, Type: DivType> SimpleElement<'re> for Div<'re, Type, WithChild> {
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
            name: self.arena.alloc("div"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
