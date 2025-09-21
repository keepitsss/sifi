//! 're = 'rendering
//!
//! Idea: implement palpable content by initializing element with pre_render_callback check

use std::{collections::HashSet, fmt::Write};

use bumpalo::{Bump, collections::Vec};

macro_rules! cx_write {
    ($output:expr, $($arg:tt)*) => {
        write!($output.output, $($arg)+).unwrap()
    };
}
macro_rules! cx_writeln {
    ($output:expr, $($arg:tt)*) => {
        writeln!($output.output, $($arg)+).unwrap()
    };
}

macro_rules! derive_pre_render_hooks {
    ($lifetime:lifetime, $ty:ty) => {
        impl<$lifetime> PreRenderHooks<$lifetime> for $ty {
            type This = Self;
            unsafe fn set_pre_render_hook(
                &mut self,
                hook: impl Fn(&Self, &mut Context) + $lifetime,
            ) {
                unsafe {
                    self.pre_render_hook.set_pre_render_hook(hook);
                }
            }
            fn get_pre_render_hook(&self) -> Option<Hook<$lifetime, Self>> {
                self.pre_render_hook.get_pre_render_hook()
            }
        }
    };
}

pub mod utils;

pub trait Renderable {
    fn render(&self, cx: &mut Context);
}

pub struct Context<'re> {
    pub indentation: utils::Indentation,
    pub output: std::string::String,
    pub arena: &'re Bump,
    pub ids: HashSet<&'re str>,
    pub styles: HashSet<&'re str>,
}

#[derive(Clone, Copy)]
pub struct AnyElement<'re>(pub &'re dyn Renderable);
impl<'re> Renderable for AnyElement<'re> {
    fn render(&self, cx: &mut Context) {
        self.0.render(cx);
    }
}
pub trait IntoElement<'re> {
    fn into_any_element(self, arena: &'re Bump) -> AnyElement<'re>;
}
impl<'re, T> IntoElement<'re> for T
where
    T: Renderable + 're,
{
    fn into_any_element(self, arena: &'re Bump) -> AnyElement<'re> {
        AnyElement(arena.alloc(self) as &dyn Renderable)
    }
}
/// https://html.spec.whatwg.org/#flow-content-2
pub trait FlowContent: Renderable {}
pub trait SectioningContent: FlowContent {}
pub trait HeadingContent: FlowContent {}
pub trait PhrasingContent: FlowContent {}
pub trait EmbeddedContent: PhrasingContent {}
pub trait InteractiveContent: FlowContent {}
pub trait MetadataContent: Renderable {}
/// # Safety
/// see docs, TLDR: should have content
pub unsafe trait PalpableContent: FlowContent {}
pub trait SelectInnerContent: Renderable {}
pub trait OptgroupInnerContent: Renderable {}
pub trait OptionInnerContent: Renderable {}

pub trait BuiltinHtmlElement: Sized {
    fn class(self, class: &str) -> Self;
    fn classes<'a>(mut self, classes: impl IntoIterator<Item = &'a str>) -> Self {
        let mut count = 0;
        for class in classes.into_iter() {
            self = self.class(class);
            count += 1;
        }
        assert_ne!(count, 0, "empty classes provided");
        assert_ne!(count, 1, "use 'class' method to provide one class");
        self
    }
    fn id(self, id: &str) -> Self;
}
macro_rules! derive_class {
    () => {
        fn class(mut self, class: &str) -> Self {
            self.classes.add(class);
            self
        }
    };
}
macro_rules! derive_id {
    () => {
        fn id(mut self, id: &str) -> Self {
            assert!(self.id.is_none());
            assert!(id.chars().all(|c| !c.is_ascii_whitespace()));
            assert!(!id.is_empty());
            self.id = Some(self.arena.alloc_str(id));
            self.with_pre_render_hook(|this, cx| {
                let id = this.id.unwrap();
                if cx.ids.contains(id) {
                    panic!("'{id}' id duplicate");
                }
                cx.ids.insert(cx.arena.alloc_str(id));
            })
        }
    };
}

mod elements;
pub use elements::*;

pub fn html(arena: &Bump) -> Html<'_> {
    Html {
        head: head(arena),
        body: None,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        arena,
    }
    .with_pre_render_hook(|_this, cx| {
        assert!(cx.output.is_empty());
        cx.output.push_str("<!DOCTYPE html>");
    })
}
fn head(arena: &Bump) -> Head<'_> {
    Head {
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        arena,
    }
}
pub fn body(arena: &Bump) -> Body<'_> {
    Body {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        arena,
    }
}

pub fn div(arena: &Bump) -> Div<'_> {
    Div {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
pub fn h(level: u8, arena: &Bump) -> Heading<'_> {
    assert!((1..=6).contains(&level));
    let mut pre_render_hook = PreRenderHookStorage::new_in(arena);
    pre_render_hook.add_pre_render_hook(|this: &Heading, _cx| {
        assert!(!this.children.is_empty());
    });
    Heading {
        level,
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook,
    }
}
pub fn p(arena: &Bump) -> Paragraph<'_> {
    Paragraph {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
pub fn a(arena: &Bump) -> Link<'_> {
    Link {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        href: None,
        download: false,
        ping: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
/// The article element represents a complete, or self-contained, composition in a document, page, application, or site and that is, in principle, independently distributable or reusable, e.g. in syndication. This could be a forum post, a magazine or newspaper article, a blog entry, a user-submitted comment, an interactive widget or gadget, or any other independent item of content.
///
/// When article elements are nested, the inner article elements represent articles that are in principle related to the contents of the outer article. For instance, a blog entry on a site that accepts user-submitted comments /// could represent the comments as article elements nested within the article element for the blog entry.
///
/// Author information associated with an article element (q.v. the address element) does not apply to nested article elements.
pub fn article(arena: &Bump) -> Article<'_> {
    // # Safety
    // Needed for palpable content.
    let mut pre_render_hook = PreRenderHookStorage::new_in(arena);
    pre_render_hook.add_pre_render_hook(|this: &Article, _cx| {
        assert!(!this.children.is_empty());
    });
    Article {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook,
    }
}
/// The section element represents a generic section of a document or application. A section, in this context, is a thematic grouping of content, typically with a heading.
pub fn section(arena: &Bump) -> Section<'_> {
    // # Safety
    // Needed for palpable content.
    let mut pre_render_hook = PreRenderHookStorage::new_in(arena);
    pre_render_hook.add_pre_render_hook(|this: &Section, _cx| {
        assert!(!this.children.is_empty());
    });
    Section {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook,
    }
}
/// The nav element represents a section of a page that links to other pages or to parts within the page: a section with navigation links.
pub fn nav(arena: &Bump) -> Navigation<'_> {
    // # Safety
    // Needed for palpable content.
    let mut pre_render_hook = PreRenderHookStorage::new_in(arena);
    pre_render_hook.add_pre_render_hook(|this: &Navigation, _cx| {
        assert!(!this.children.is_empty());
    });
    Navigation {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook,
    }
}
/// The aside element represents a section of a page that consists of content that is tangentially related to the content around the aside element, and which could be considered separate from that content. Such sections are often represented as sidebars in printed typography.
///
/// The element can be used for typographical effects like pull quotes or sidebars, for advertising, for groups of nav elements, and for other content that is considered separate from the main content of the page.
pub fn aside(arena: &Bump) -> Aside<'_> {
    // # Safety
    // Needed for palpable content.
    let mut pre_render_hook = PreRenderHookStorage::new_in(arena);
    pre_render_hook.add_pre_render_hook(|this: &Aside, _cx| {
        assert!(!this.children.is_empty());
    });
    Aside {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook,
    }
}
// TODO: header
// TODO: footer
const _: () = ();
