//! 're = 'rendering
//!
//! # Headings and outlines
//! `h1`–`h6` elements have a heading level, which is given by getting the element's computed heading level.
//!
//! These elements represent headings. The lower a heading's heading level is, the fewer ancestor sections the heading has.
//!
//! The outline is all headings in a document, in tree order.
//!
//! The outline should be used for generating document outlines, for example when generating tables of contents. When creating an interactive table of contents, entries should jump the user to the relevant heading.
//!
//! If a document has one or more headings, at least a single heading within the outline should have a heading level of 1.
//!
//! Each heading following another heading lead in the outline must have a heading level that is less than, equal to, or 1 greater than lead's heading level.

use std::{collections::HashSet, fmt::Write, marker::PhantomData};

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
    ($lifetime:lifetime $(+ $generic:tt)*, $ty:ty) => {
        impl<$lifetime $(,$generic)*> PreRenderHooks<$lifetime> for $ty {
            type This = Self;
            unsafe fn set_pre_render_hook(
                &mut self,
                hook: impl Fn(&Self::This, &mut Context) + $lifetime,
            ) {
                unsafe {
                    self.pre_render_hook.set_pre_render_hook(hook);
                }
            }
            fn get_pre_render_hook(&self) -> Option<Hook<$lifetime, Self::This>> {
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

pub mod elements;
use elements::*;
pub use elements::{NoValue, OrderedListMarkerType, WithValue};
// pub use elements::*;

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

/// The `div` element has no special meaning at all.
/// It represents its children.
/// It can be used with the `class`, `lang`, and `title` attributes to mark up semantics common to a group of consecutive elements.
/// It can also be used in a `dl` element, wrapping groups of `dt` and `dd` elements.
///
/// > ![NOTE]
/// >
/// > Authors are strongly encouraged to view the `div` element as an element of last resort, for when no other element is suitable.
/// > Use of more appropriate elements instead of the `div` element leads to better accessibility for readers and easier maintainability for authors.
pub fn div<Type: DivType>(arena: &Bump) -> Div<'_, Type, WithoutChild> {
    Div {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        marker: PhantomData,
        has_child: PhantomData,
    }
}
// TODO: docs
pub fn h(level: u8, arena: &Bump) -> Heading<'_, WithoutChild> {
    assert!((1..=6).contains(&level));
    Heading {
        level,
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `hgroup` element represents a heading and related content. The element may be used to group an h1–h6 element with one or more p elements containing content representing a subheading, alternative title, or tagline.
pub fn hgroup(arena: &Bump) -> HeadingGroup<'_, WithoutHeader> {
    HeadingGroup {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_heading: PhantomData,
    }
}
/// The `p` element represents a paragraph.
///
/// The `p` element should not be used when a more specific element is more appropriate.
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
/// The `article` element represents a complete, or self-contained, composition in a document, page, application, or site and that is, in principle, independently distributable or reusable, e.g. in syndication.
/// This could be a forum post, a magazine or newspaper `article`, a blog entry, a user-submitted comment, an interactive widget or gadget, or any other independent item of content.
///
/// When `article` elements are nested, the inner `article` elements represent `articles` that are in principle related to the contents of the outer `article`.
/// For instance, a blog entry on a site that accepts user-submitted comments could represent the comments as `article` elements nested within the `article` element for the blog entry.
///
/// Author information associated with an `article` element (q.v. the address element) does not apply to nested `article` elements.
pub fn article(arena: &Bump) -> Article<'_, WithoutChild> {
    Article {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `section` element represents a generic `section` of a document or application. A `section`, in this context, is a thematic grouping of content, typically with a heading.
pub fn section(arena: &Bump) -> Section<'_, WithoutChild> {
    Section {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `nav` element represents a section of a page that links to other pages or to parts within the page: a section with navigation links.
pub fn nav(arena: &Bump) -> Navigation<'_, WithoutChild> {
    Navigation {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `aside` element represents a section of a page that consists of content that is tangentially related to the content around the `aside` element, and which could be considered separate from that content.
/// Such sections are often represented as sidebars in printed typography.
///
/// The element can be used for typographical effects like pull quotes or sidebars, for advertising, for groups of nav elements, and for other content that is considered separate from the main content of the page.
pub fn aside(arena: &Bump) -> Aside<'_, WithoutChild> {
    Aside {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The header element represents a group of introductory or navigational aids.
pub fn header(arena: &Bump) -> Header<'_, WithoutChild> {
    Header {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `footer` element represents a `footer` for its nearest ancestor sectioning content element, or for the body element if there is no such ancestor.
/// A `footer` typically contains information about its section such as who wrote it, links to related documents, copyright data, and the like.
///
/// When the `footer` element contains entire sections, they represent appendices, indices, long colophons, verbose license agreements, and other such content.
///
/// `Footers` don't necessarily have to appear at the end of a section, though they usually do.
///
/// When there is no ancestor sectioning content element, then it applies to the whole page.
pub fn footer(arena: &Bump) -> Footer<'_, WithoutChild> {
    Footer {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `address` element represents the contact information for its nearest article or body element ancestor.
/// If that is the body element, then the contact information applies to the document as a whole.
///
/// The `address` element must not be used to represent arbitrary addresses (e.g. postal addresses), unless those addresses are in fact the relevant contact information.
/// (The `p` element is the appropriate element for marking up postal addresses in general.)
///
/// The `address` element must not contain information other than contact information.
///
/// Typically, the `address` element would be included along with other information in a footer element.
pub fn address(arena: &Bump) -> Address<'_, WithoutChild> {
    Address {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}

/// The `hr` element represents a paragraph-level thematic break, e.g., a scene change in a story, or a transition to another topic within a section of a reference book;
/// alternatively, it represents a separator between a set of options of a `select` element.
pub fn hr(arena: &Bump) -> ThematicBreak<'_> {
    ThematicBreak {
        classes: Classes::new_in(arena),
        id: None,
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}
/// The `pre` element represents a block of preformatted text, in which structure is represented by typographic conventions rather than by elements.
///
/// Some examples of cases where the pre element could be used:
/// - Including an email, with paragraphs indicated by blank lines, lists indicated by lines prefixed with a bullet, and so on.
/// - Including fragments of computer code, with structure indicated according to the conventions of that language.
/// - Displaying ASCII art.
pub fn pre(arena: &Bump) -> PreformattedText<'_, WithoutChild> {
    PreformattedText {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
/// The `blockquote` element represents a section that is quoted from another source.
///
/// Content inside a `blockquote` must be quoted from another source, whose address, if it has one, may be cited in the cite attribute.
///
/// If the cite attribute is present, it must be a valid URL potentially surrounded by spaces.
/// User agents may allow users to follow such citation links, but they are primarily intended for private use
/// (e.g., by server-side scripts collecting statistics about a site's use of quotations), not for readers.
///
/// The content of a `blockquote` may be abbreviated or may have context added in the conventional manner for the text's language.
pub fn blockquote(arena: &Bump) -> BlockQuote<'_, WithoutChild> {
    BlockQuote {
        classes: Classes::new_in(arena),
        id: None,
        cite: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}

/// The `ol` element represents a list of items, where the items have been intentionally ordered, such that changing the order would change the meaning of the document.
pub fn ol(arena: &Bump) -> OrderedList<'_> {
    OrderedList {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        reversed: false,
        starting_value: None,
        marker_type: None,
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}

/// The `ul` element represents a list of items, where the order of the items is not important — that is, where changing the order would not materially change the meaning of the document.
pub fn ul(arena: &Bump) -> UnorderedList<'_> {
    UnorderedList {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}

/// The `menu` element represents a toolbar consisting of its contents, in the form of an unordered list of items (represented by `li` elements), each of which represents a command that the user can perform or activate.
pub fn menu(arena: &Bump) -> Menu<'_> {
    Menu {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}

/// The `li` element represents a list item.
#[allow(private_bounds)]
pub fn li<Value: ListItemValueProp>(arena: &Bump, value: Value) -> ListItem<'_, Value> {
    ListItem {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        value,
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}

/// The `figure` element represents some flow content, optionally with a caption, that is self-contained (like a complete sentence)
/// and is typically referenced as a single unit from the main flow of the document.
///
/// "Self-contained" in this context does not necessarily mean independent.
/// For example, each sentence in a paragraph is self-contained;
/// an image that is part of a sentence would be inappropriate for `figure`, but an entire sentence made of images would be fitting.
///
/// The element can thus be used to annotate illustrations, diagrams, photos, code listings, etc.
///
/// The `figcaption` element child of the element, if any, represents the caption of the `figure` element's contents.
/// If there is no child `figcaption` element, then there is no caption.
///
/// A `figure` element's contents are part of the surrounding flow.
/// If the purpose of the page is to display the figure, for example a photograph on an image sharing site,
/// the `figure` and `figcaption` elements can be used to explicitly provide a caption for that figure.
/// For content that is only tangentially related, or that serves a separate purpose than the surrounding flow,
/// the `aside` element should be used (and can itself wrap a figure).
/// For example, a pull quote that repeats content from an `article` would be more appropriate in an `aside` than in a figure,
/// because it isn't part of the content, it's a repetition of the content for the purposes of enticing readers or highlighting key topics.
pub fn figure(arena: &Bump) -> Figure<'_, EmptyState> {
    Figure {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        marker: PhantomData,
    }
}
/// The `figcaption` element represents a caption or legend for the rest of the contents of the `figcaption` element's parent `figure` element.
pub fn figcaption(arena: &Bump) -> FigureCaption<'_> {
    FigureCaption {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
    }
}

/// The `main` element represents the dominant contents of the document.
///
/// # Safety
/// - A document must not have more than one `main` element that does not have the hidden attribute specified.
/// - Ancestor elements must be limited to `html`, `body`, `div`, `form` without an accessible name, and autonomous custom elements.
pub unsafe fn html_main(arena: &Bump) -> Main<'_, WithoutChild> {
    Main {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}

/// The `search` element represents a part of a document or application that contains a set of form controls or other content related to performing a search or filtering operation.
/// This could be a search of the web site or application; a way of searching or filtering search results on the current web page; or a global or Internet-wide search function.
///
/// > ![NOTE]
/// >
/// > It's not appropriate to use the `search` element just for presenting search results, though suggestions and links as part of "quick search" results can be included as part of a search feature.
/// > Rather, a returned web page of search results would instead be expected to be presented as part of the main content of that web page.
pub fn search(arena: &Bump) -> Search<'_, WithoutChild> {
    Search {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}

/// Represents stress emphasis of its contents.
///
/// The level of stress that a particular piece of content has is given by its number of ancestor `em` elements.
///
/// **The placement of stress emphasis changes the meaning of the sentence.**
/// The element thus forms an integral part of the content.
/// The precise way in which stress is used in this way depends on the language.
pub fn em(arena: &Bump) -> Emphasis<'_, WithoutChild> {
    Emphasis {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}

/// Represents strong importance, seriousness, or urgency for its contents.
///
/// *Importance*: the `strong` element can be used in a heading, caption, or paragraph to distinguish the part that really matters from other parts that might be more detailed, more jovial, or merely boilerplate.
///     (This is distinct from marking up subheadings, for which the hgroup element is appropriate.)
///
/// For example, the first word of the previous paragraph is marked up with `strong` to distinguish it from the more detailed text in the rest of the paragraph.
///
/// *Seriousness*: the `strong` element can be used to mark up a warning or caution notice.
///
/// *Urgency*: the `strong` element can be used to denote contents that the user needs to see sooner than other parts of the document.
///
/// The relative level of importance of a piece of content is given by its number of ancestor `strong` elements; each `strong` element increases the importance of its contents.
///
/// **Changing the importance of a piece of text with the `strong` element does not change the meaning of the sentence.**
pub fn strong(arena: &Bump) -> StrongImportance<'_, WithoutChild> {
    StrongImportance {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}

/// Represents side comments such as small print.
///
/// > Small print typically features disclaimers, caveats, legal restrictions, or copyrights.
/// > Small print is also sometimes used for attribution, or for satisfying licensing requirements.
///
/// > The `small` element does not "de-emphasize" or lower the importance of text emphasized by the em element or marked as important with the `strong` element.
/// > To mark text as not emphasized or important, simply do not mark it up with the em or `strong` elements respectively.
///
/// The `small` element should not be used for extended spans of text, such as multiple paragraphs, lists, or sections of text. It is only intended for short runs of text.
/// The text of a page listing terms of use, for instance, would not be a suitable candidate for the `small` element:
///     in such a case, the text is not a side comment, it is the main content of the page.
///
/// The `small` element must not be used for subheadings; for that purpose, use the `hgroup` element.
pub fn small(arena: &Bump) -> SmallPrint<'_, WithoutChild> {
    SmallPrint {
        classes: Classes::new_in(arena),
        id: None,
        children: Vec::new_in(arena),
        arena,
        pre_render_hook: PreRenderHookStorage::new_in(arena),
        has_child: PhantomData,
    }
}
