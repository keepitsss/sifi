use std::marker::PhantomData;

use super::*;

mod attributes;
pub(crate) use attributes::*;

mod generic_html_element;
use generic_html_element::*;

mod hooks;
pub(crate) use hooks::*;

pub struct WithChild;
pub struct WithoutChild;

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

mod div;
pub use div::*;

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

// NOTE: you can use trait, that is implemented only for children

// TODO

// ===== links =====

mod link;
pub use link::*;

// TODO

// text-level semantics
// TODO
