use super::*;

pub struct EmptyState;
pub struct WithContent;
impl CorrectFigureState for WithContent {}
impl From<WithContent> for GenericState {
    fn from(_: WithContent) -> Self {
        GenericState
    }
}
pub struct WithCaption;
pub struct WithCaptionAndContent;
impl CorrectFigureState for WithCaptionAndContent {}
impl From<WithCaptionAndContent> for GenericState {
    fn from(_: WithCaptionAndContent) -> Self {
        GenericState
    }
}
pub struct WithContentAndCaption;
impl CorrectFigureState for WithContentAndCaption {}
impl From<WithContentAndCaption> for GenericState {
    fn from(_: WithContentAndCaption) -> Self {
        GenericState
    }
}
pub struct GenericState;
trait CorrectFigureState: Into<GenericState> {}

impl<'re, State: CorrectFigureState> From<&Figure<'re, State>> for &Figure<'re, GenericState> {
    fn from(value: &Figure<'re, State>) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

pub struct Figure<'re, State> {
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

impl<'re, State: 're> BuiltinHtmlElement for Figure<'re, State> {
    derive_class!();
    derive_id!();
}
impl<'re, State: 're> PreRenderHooks<'re> for Figure<'re, State> {
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
impl<'re, State> Figure<'re, State> {
    unsafe fn change_state<NewState>(self) -> Figure<'re, NewState> {
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
impl<'re, State: CorrectFigureState + 're> FlowContent for Figure<'re, State> {}
impl<'re, State: CorrectFigureState + 're> SimpleElement<'re> for Figure<'re, State> {
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
