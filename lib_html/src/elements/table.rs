use super::*;

#[allow(private_bounds)]
pub trait TableState: Marker {}
pub trait CorrectTableState: TableState {}
pub struct Empty;
pub struct WithCaption;
pub struct WithColumnGroup;
pub struct WithHeader;
pub struct WithBodies;
pub struct WithRows;
pub struct WithFooter;
impl Marker for Empty {}
impl Marker for WithCaption {}
impl Marker for WithColumnGroup {}
impl Marker for WithHeader {}
impl Marker for WithBodies {}
impl Marker for WithRows {}
impl Marker for WithFooter {}
impl TableState for Empty {}
impl TableState for WithCaption {}
impl TableState for WithColumnGroup {}
impl TableState for WithHeader {}
impl TableState for WithBodies {}
impl TableState for WithRows {}
impl TableState for WithFooter {}
impl CorrectTableState for WithCaption {}
impl CorrectTableState for WithColumnGroup {}
impl CorrectTableState for WithHeader {}
impl CorrectTableState for WithBodies {}
impl CorrectTableState for WithRows {}
impl CorrectTableState for WithFooter {}

impl TableState for GenericState {}

pub struct Table<'re, State: TableState> {
    pub classes: Classes<'re>,
    pub id: Option<&'re str>,
    pub children: Vec<'re, AnyElement<'re>>,
    pub(crate) arena: &'re Bump,
    pub(crate) pre_render_hook: PreRenderHookStorage<'re, Table<'re, GenericState>>,
    pub(crate) state: PhantomData<State>,
}
impl<'re, State: CorrectTableState> From<&Table<'re, State>> for &Table<'re, GenericState> {
    fn from(value: &Table<'re, State>) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
impl<State: TableState> BuiltinHtmlElement for Table<'_, State> {
    derive_class!();
    derive_id!();
}
impl<'re, State: TableState> PreRenderHooks<'re> for Table<'re, State> {
    type This = Table<'re, GenericState>;
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're) {
        unsafe {
            self.pre_render_hook.set_pre_render_hook(hook);
        }
    }
    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>> {
        self.pre_render_hook.get_pre_render_hook()
    }
}
impl<State: TableState> FlowContent for Table<'_, State> where State: CorrectTableState {}
// # Safety
// Typesafe design
unsafe impl<State: TableState> PalpableContent for Table<'_, State> where State: CorrectTableState {}

impl<'re, State: TableState> SimpleElement<'re> for Table<'re, State>
where
    State: CorrectTableState,
{
    type GenericSelf = Table<'re, GenericState>;
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
            name: self.arena.alloc("table"),
            attributes: attrs.into_bump_slice(),
            children: strip_anyelement(self.arena, &self.children),
            late_children: &[],
        }
    }
}

impl<'re, OldState: TableState> Table<'re, OldState> {
    unsafe fn change_state<NewState: TableState>(self) -> Table<'re, NewState> {
        let Table {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            state: _,
        } = self;
        Table {
            classes,
            id,
            children,
            arena,
            pre_render_hook,
            state: PhantomData,
        }
    }
    unsafe fn with<NewState: TableState>(
        mut self,
        children: impl IntoElement<'re>,
    ) -> Table<'re, NewState> {
        self.children.push(children.into_any_element(self.arena));
        unsafe { self.change_state() }
    }
}
impl<'re> Table<'re, Empty> {
    pub fn caption(self, caption: Caption<'re>) -> Table<'re, WithCaption> {
        unsafe { self.with(caption) }
    }
    pub fn header(self, header: TableHeader<'re>) -> Table<'re, WithHeader> {
        unsafe { self.with(header) }
    }
    pub fn body(self, table_body: TableBody<'re>) -> Table<'re, WithBodies> {
        unsafe { self.with(table_body) }
    }
    pub fn row(self, table_row: TableRow<'re>) -> Table<'re, WithRows> {
        unsafe { self.with(table_row) }
    }
    pub fn bodies(
        mut self,
        table_bodies: impl IntoIterator<Item = TableBody<'re>>,
    ) -> Table<'re, WithRows> {
        for body in table_bodies {
            self.children.push(body.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn rows(
        mut self,
        table_rows: impl IntoIterator<Item = TableRow<'re>>,
    ) -> Table<'re, WithRows> {
        for row in table_rows {
            self.children.push(row.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn footer(self, footer: TableFooter<'re>) -> Table<'re, WithFooter> {
        unsafe { self.with(footer) }
    }
}
impl<'re> Table<'re, WithCaption> {
    pub fn header(self, header: TableHeader<'re>) -> Table<'re, WithHeader> {
        unsafe { self.with(header) }
    }
    pub fn body(self, table_body: TableBody<'re>) -> Table<'re, WithBodies> {
        unsafe { self.with(table_body) }
    }
    pub fn row(self, table_row: TableRow<'re>) -> Table<'re, WithRows> {
        unsafe { self.with(table_row) }
    }
    pub fn bodies(
        mut self,
        table_bodies: impl IntoIterator<Item = TableBody<'re>>,
    ) -> Table<'re, WithRows> {
        for body in table_bodies {
            self.children.push(body.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn rows(
        mut self,
        table_rows: impl IntoIterator<Item = TableRow<'re>>,
    ) -> Table<'re, WithRows> {
        for row in table_rows {
            self.children.push(row.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn footer(self, footer: TableFooter<'re>) -> Table<'re, WithFooter> {
        unsafe { self.with(footer) }
    }
}
impl<'re> Table<'re, WithColumnGroup> {
    pub fn header(self, header: TableHeader<'re>) -> Table<'re, WithHeader> {
        unsafe { self.with(header) }
    }
    pub fn body(self, table_body: TableBody<'re>) -> Table<'re, WithBodies> {
        unsafe { self.with(table_body) }
    }
    pub fn row(self, table_row: TableRow<'re>) -> Table<'re, WithRows> {
        unsafe { self.with(table_row) }
    }
    pub fn bodies(
        mut self,
        table_bodies: impl IntoIterator<Item = TableBody<'re>>,
    ) -> Table<'re, WithRows> {
        for body in table_bodies {
            self.children.push(body.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn rows(
        mut self,
        table_rows: impl IntoIterator<Item = TableRow<'re>>,
    ) -> Table<'re, WithRows> {
        for row in table_rows {
            self.children.push(row.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn footer(self, footer: TableFooter<'re>) -> Table<'re, WithFooter> {
        unsafe { self.with(footer) }
    }
}
impl<'re> Table<'re, WithHeader> {
    pub fn body(self, table_body: TableBody<'re>) -> Table<'re, WithBodies> {
        unsafe { self.with(table_body) }
    }
    pub fn row(self, table_row: TableRow<'re>) -> Table<'re, WithRows> {
        unsafe { self.with(table_row) }
    }
    pub fn bodies(
        mut self,
        table_bodies: impl IntoIterator<Item = TableBody<'re>>,
    ) -> Table<'re, WithRows> {
        for body in table_bodies {
            self.children.push(body.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn rows(
        mut self,
        table_rows: impl IntoIterator<Item = TableRow<'re>>,
    ) -> Table<'re, WithRows> {
        for row in table_rows {
            self.children.push(row.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn footer(self, footer: TableFooter<'re>) -> Table<'re, WithFooter> {
        unsafe { self.with(footer) }
    }
}
impl<'re> Table<'re, WithBodies> {
    pub fn body(self, table_body: TableBody<'re>) -> Self {
        unsafe { self.with(table_body) }
    }
    pub fn bodies(
        mut self,
        table_bodies: impl IntoIterator<Item = TableBody<'re>>,
    ) -> Table<'re, WithRows> {
        for body in table_bodies {
            self.children.push(body.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn footer(self, footer: TableFooter<'re>) -> Table<'re, WithFooter> {
        unsafe { self.with(footer) }
    }
}
impl<'re> Table<'re, WithRows> {
    pub fn row(self, table_row: TableRow<'re>) -> Self {
        unsafe { self.with(table_row) }
    }
    pub fn rows(
        mut self,
        table_rows: impl IntoIterator<Item = TableRow<'re>>,
    ) -> Table<'re, WithRows> {
        for row in table_rows {
            self.children.push(row.into_any_element(self.arena));
        }
        unsafe { self.change_state() }
    }
    pub fn footer(self, footer: TableFooter<'re>) -> Table<'re, WithFooter> {
        unsafe { self.with(footer) }
    }
}
