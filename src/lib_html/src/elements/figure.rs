use super::*;

#[allow(private_bounds)]
pub trait FigureState: Marker {}
trait CorrectFigureState: FigureState {}

pub struct EmptyState;
impl Marker for EmptyState {}
impl FigureState for EmptyState {}
pub struct WithContent;
impl Marker for WithContent {}
impl FigureState for WithContent {}
impl CorrectFigureState for WithContent {}
pub struct WithCaption;
impl Marker for WithCaption {}
impl FigureState for WithCaption {}
pub struct WithCaptionAndContent;
impl Marker for WithCaptionAndContent {}
impl FigureState for WithCaptionAndContent {}
impl CorrectFigureState for WithCaptionAndContent {}
pub struct WithContentAndCaption;
impl Marker for WithContentAndCaption {}
impl FigureState for WithContentAndCaption {}
impl CorrectFigureState for WithContentAndCaption {}

impl FigureState for GenericState {}

impl<'re, State: CorrectFigureState> From<&Figure<'re, State>> for &Figure<'re, GenericState> {
    fn from(value: &Figure<'re, State>) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

pub struct Figure<'re, State: FigureState> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Figure<'re, GenericState>>,
    pub(crate) marker: PhantomData<State>,
}
pub struct FigureCaption<'re> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Self>,
}

impl<'re, State: FigureState> BuiltinHtmlElement for Figure<'re, State> {
    derive_class!();
    derive_id!();
}
impl<'re, State: FigureState> PreRenderHooks<'re> for Figure<'re, State> {
    type This = Figure<'re, GenericState>;
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>> {
        self.pre_render_hook.get_pre_render_hook()
    }
}
impl<'re, State: FigureState> Figure<'re, State> {
    unsafe fn change_state<NewState: FigureState>(self) -> Figure<'re, NewState> {
        let Figure {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            marker: _,
        } = self;
        Figure {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            marker: PhantomData,
        }
    }
}
impl<'re> Figure<'re, EmptyState> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Figure<'re, WithContent> {
        self.children.push(child.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
    pub fn caption(mut self, caption: FigureCaption<'re>) -> Figure<'re, WithCaption> {
        self.children.push(caption.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
}
impl<'re> Figure<'re, WithContent> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Figure<'re, WithContent> {
        self.children.push(child.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
    pub fn caption(mut self, caption: FigureCaption<'re>) -> Figure<'re, WithContentAndCaption> {
        self.children.push(caption.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
}
impl<'re> Figure<'re, WithCaption> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Figure<'re, WithCaptionAndContent> {
        self.children.push(child.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
}
impl<'re> Figure<'re, WithCaptionAndContent> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Figure<'re, WithCaptionAndContent> {
        self.children.push(child.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
}
impl<'re, State: CorrectFigureState> FlowContent for Figure<'re, State> {}
impl<'re, State: CorrectFigureState> SimpleElement<'re> for Figure<'re, State> {
    type GenericSelf = Figure<'re, GenericState>;
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
            name: self.arena.alloc("figure"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

impl BuiltinHtmlElement for FigureCaption<'_> {
    derive_class!();
    derive_id!();
}
derive_pre_render_hooks!('re, FigureCaption<'re>);
impl<'re> FigureCaption<'re> {
    pub fn child(mut self, child: impl FlowContent + 're) -> Self {
        self.children.push(child.into_any_element(self.arena));
        self
    }
}
impl<'re> SimpleElement<'re> for FigureCaption<'re> {
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
            name: self.arena.alloc("figcaption"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}
