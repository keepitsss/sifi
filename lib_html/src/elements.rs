use std::marker::PhantomData;

use super::*;

mod attributes;
pub(crate) use attributes::*;

mod generic_html_element;
use generic_html_element::*;

mod hooks;
pub(crate) use hooks::*;

trait Marker: 'static {}

pub struct GenericState;
impl Marker for GenericState {}

#[allow(private_bounds)]
pub trait ChildExistenceState: Marker {}
pub struct WithChild;
pub struct WithoutChild;
impl Marker for WithChild {}
impl ChildExistenceState for WithChild {}
impl Marker for WithoutChild {}
impl ChildExistenceState for WithoutChild {}

// ===== text element =====
mod string;

// ===== document element =====
mod html;
pub use html::*;

// ===== metadata elements =====

mod head;
pub use head::*;

// ===== section elements =====

mod body;
pub use body::*;

mod article;
pub use article::*;

mod section;
pub use section::*;

mod navigation;
pub use navigation::*;

mod aside;
pub use aside::*;

mod heading;
pub use heading::*;

mod heading_group;
pub use heading_group::*;

// TODO: header can't contain other header or footer
mod header;
pub use header::*;

// TODO: footer can't contain other header or footer
mod footer;
pub use footer::*;

mod address;
pub use address::*;

// ===== grouping elements =====

mod paragraph;
pub use paragraph::*;

mod thematic_break;
pub use thematic_break::*;

mod preformatted_text;
pub use preformatted_text::*;

mod block_quote;
pub use block_quote::*;

mod ordered_list;
pub use ordered_list::*;

mod unordered_list;
pub use unordered_list::*;

mod menu;
pub use menu::*;

mod list_item;
pub use list_item::*;

// TODO: description list

mod figure;
pub use figure::*;

mod main;
pub use main::*;

mod search;
pub use search::*;

mod div;
pub use div::*;

// ===== text-level =====

mod link;
pub use link::*;

mod emphasis;
pub use emphasis::*;

mod strong_importance;
pub use strong_importance::*;

mod small_print;
pub use small_print::*;

mod stale;
pub use stale::*;

mod cite;
pub use cite::*;

mod quote;
pub use quote::*;

mod code;
pub use code::*;

mod program_sample;
pub use program_sample::*;

mod keyboard_input;
pub use keyboard_input::*;

mod alternate;
pub use alternate::*;

mod meaningless_attention;
pub use meaningless_attention::*;

mod mark;
pub use mark::*;

// TODO: dfn, abbr - definitions

// TODO: span

mod line_break;
pub use line_break::*;

// ===== table =====

mod table {
    use super::*;

    #[allow(private_bounds)]
    pub trait TableState: Marker {}
    trait CorrectTableState: TableState {}
    pub struct Empty;
    pub struct WithCaption;
    pub struct WithColumnGroup;
    pub struct WithHead;
    pub struct WithBody;
    pub struct WithRows;
    pub struct WithFooter;
    impl Marker for Empty {}
    impl Marker for WithCaption {}
    impl Marker for WithColumnGroup {}
    impl Marker for WithHead {}
    impl Marker for WithBody {}
    impl Marker for WithRows {}
    impl Marker for WithFooter {}
    impl TableState for Empty {}
    impl TableState for WithCaption {}
    impl TableState for WithColumnGroup {}
    impl TableState for WithHead {}
    impl TableState for WithBody {}
    impl TableState for WithRows {}
    impl TableState for WithFooter {}
    impl CorrectTableState for WithCaption {}
    impl CorrectTableState for WithColumnGroup {}
    impl CorrectTableState for WithHead {}
    impl CorrectTableState for WithBody {}
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
    impl<State: CorrectTableState> BuiltinHtmlElement for Table<'_, State> {
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
    }
    impl<'re> Table<'re, Empty> {
        pub fn caption(mut self, caption: Caption<'re>) -> Table<'re, WithCaption> {
            self.children.push(caption.into_any_element(self.arena));
            unsafe { self.change_state() }
        }
    }
}

mod caption;
pub use caption::*;
