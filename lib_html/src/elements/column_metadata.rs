use super::*;

pub struct WithCol;
pub struct WithSpan;
pub trait TableColumnsState: 'static {}
impl TableColumnsState for Empty {}
impl TableColumnsState for WithCol {}
impl TableColumnsState for WithSpan {}
impl<'re, State: TableColumnsState> From<&TableColumnGroup<'re, State>>
    for &TableColumnGroup<'re, GenericState>
{
    fn from(value: &TableColumnGroup<'re, State>) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
impl<'re, State: TableColumnsState> From<&TableColumns<'re, State>>
    for &TableColumns<'re, GenericState>
{
    fn from(value: &TableColumns<'re, State>) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

pub struct Span(pub Option<usize>);
impl<'re> Attribute<'re> for Span {
    fn new_in(_arena: &'re Bump) -> Self {
        Span(None)
    }
    fn render(&self) -> Option<HtmlAttribute<'re>> {
        self.0.map(|x| HtmlAttribute {
            name: "span",
            value: HtmlValue::Number(x as i32),
        })
    }
}
impl Span {
    fn set(&mut self, value: usize) {
        assert!(value > 0);
        assert!(value <= 1000);
        self.0 = Some(value);
    }
}

pub struct TableColumnGroup<'re, State> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub span: Span,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, TableColumnGroup<'re, GenericState>>,
    pub(crate) state: PhantomData<State>,
}
impl<State: TableColumnsState> BuiltinHtmlElement for TableColumnGroup<'_, State> {
    derive_class!();
    derive_id!();
}
impl<'re, State: TableColumnsState> PreRenderHooks<'re> for TableColumnGroup<'re, State> {
    type This = TableColumnGroup<'re, GenericState>;
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>> {
        self.pre_render_hook.get_pre_render_hook()
    }
}
impl<'re, OldState: TableColumnsState> TableColumnGroup<'re, OldState> {
    unsafe fn change_state<NewState: TableColumnsState>(self) -> TableColumnGroup<'re, NewState> {
        let TableColumnGroup {
            classes,
            id,
            children,
            span,
            arena,
            pre_render_hook,
            state: _,
        } = self;
        TableColumnGroup {
            classes,
            id,
            children,
            span,
            arena,
            pre_render_hook,
            state: PhantomData,
        }
    }
}
impl<'re> TableColumnGroup<'re, Empty> {
    pub fn col(mut self, child: TableRow<'re>) -> TableColumnGroup<'re, WithCol> {
        self.children.push(child.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
    pub fn cols<State: TableColumnsState>(
        mut self,
        children: impl IntoIterator<Item = TableColumns<'re, State>>,
    ) -> TableColumnGroup<'re, WithCol> {
        let mut count = 0;
        for child in children {
            count += 1;
            self.children.push(child.into_any_element(self.arena));
        }
        assert!(count > 0, "You should provide at least one element");
        unsafe { self.change_state() }
    }
    pub fn span(mut self, value: usize) -> TableColumnGroup<'re, WithSpan> {
        self.span.set(value);
        unsafe { self.change_state() }
    }
}
impl<'re> TableColumnGroup<'re, WithCol> {
    pub fn col(mut self, child: TableRow<'re>) -> TableColumnGroup<'re, WithCol> {
        self.children.push(child.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
    pub fn cols(
        mut self,
        children: impl IntoIterator<Item = TableRow<'re>>,
    ) -> TableColumnGroup<'re, WithCol> {
        let mut count = 0;
        for child in children {
            count += 1;
            self.children.push(child.into_any_element(self.arena));
        }
        assert!(count > 0, "You should provide at least one element");
        unsafe { self.change_state() }
    }
}
impl<'re, State: TableColumnsState> SimpleElement<'re> for TableColumnGroup<'re, State> {
    type GenericSelf = TableColumnGroup<'re, GenericState>;
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
        if let Some(attr) = self.span.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: self.arena.alloc("colgroup"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

pub struct TableColumns<'re, State> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub span: Span,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, TableColumns<'re, GenericState>>,
    pub(crate) state: PhantomData<State>,
}
impl<State: TableColumnsState> BuiltinHtmlElement for TableColumns<'_, State> {
    derive_class!();
    derive_id!();
}
impl<'re, State: TableColumnsState> PreRenderHooks<'re> for TableColumns<'re, State> {
    type This = TableColumns<'re, GenericState>;
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>> {
        self.pre_render_hook.get_pre_render_hook()
    }
}
impl<'re> TableColumns<'re, Empty> {
    pub fn span(mut self, value: usize) -> TableColumns<'re, WithSpan> {
        self.span.set(value);
        unsafe { self.change_state() }
    }
}
impl<'re, OldState: TableColumnsState> TableColumns<'re, OldState> {
    unsafe fn change_state<NewState: TableColumnsState>(self) -> TableColumns<'re, NewState> {
        let TableColumns {
            classes,
            id,
            span,
            arena,
            pre_render_hook,
            state: _,
        } = self;
        TableColumns {
            classes,
            id,
            span,
            arena,
            pre_render_hook,
            state: PhantomData,
        }
    }
}
impl<'re, State: TableColumnsState> SimpleElement<'re> for TableColumns<'re, State> {
    type GenericSelf = TableColumns<'re, GenericState>;
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
        if let Some(attr) = self.classes.render() {
            attrs.push(attr);
        }
        GenericHtmlElement {
            name: self.arena.alloc("col"),
            attributes: attrs.into_bump_slice(),
            children: &[],
            late_children: &[],
        }
    }
}
