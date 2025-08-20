#[derive(Debug)]
struct Markdown<'a> {
    blocks: Vec<MarkdownBlock<'a>>,
}

#[derive(Debug)]
enum MarkdownBlock<'a> {
    /// A block quote marker, consists of (a) the character > together with a following space of indentation.
    ///
    /// - Basic case. If a string of lines Ls constitute a sequence of blocks Bs, then the result of prepending a block quote marker to the beginning of each line in Ls is a block quote containing Bs.
    /// - Laziness. If a string of lines Ls constitute a block quote with contents Bs, then the result of deleting the initial block quote marker from one or more lines in which the next character other than a space after the block quote marker is paragraph continuation text is a block quote with Bs as its content. Paragraph continuation text is text that will be parsed as part of the content of a paragraph, but does not occur at the beginning of the paragraph.
    /// - Consecutiveness. A document cannot contain two block quotes in a row unless there is a blank line between them.
    BlockQuote { content: Markdown<'a> },
    /// A bullet list marker is a -, +, or * character.
    ///
    /// Basic case. If a sequence of lines Ls constitute a sequence of blocks Bs starting with a character other than a space, and M is a list marker of width W followed by one space of indentation, then the result of prepending M and the following spaces to the first line of Ls, and indenting subsequent lines of Ls by W + 1 spaces, is a list item with Bs as its contents.
    ///
    /// If any line is a thematic break then that line is not a list item.
    BulletList { items: Vec<Markdown<'a>> },
    /// An ordered list marker is a sequence of 1–9 arabic digits (0-9), followed by either a . character or a ) character. (The reason for the length limit is that with 10 digits we start seeing integer overflows in some browsers.)
    ///
    /// Basic case. If a sequence of lines Ls constitute a sequence of blocks Bs starting with a character other than a space, and M is a list marker of width W followed by one space of indentation, then the result of prepending M and the following spaces to the first line of Ls, and indenting subsequent lines of Ls by W + 1 spaces, is a list item with Bs as its contents.
    OrderedList { items: Vec<(u32, Markdown<'a>)> },
    /// A line consisting of sequence of three or more matching -, _, or * characters forms a thematic break.
    LineBreak,
    /// An heading consists of a string of characters, parsed as inline content, between an opening sequence of 1–6 unescaped # characters. The opening sequence of # characters must be followed by spaces, or by the end of line. The opening # character must be preceded by one space. The raw contents of the heading are stripped of leading and trailing space before being parsed as inline content. The heading level is equal to the number of # characters in the opening sequence.
    Heading { level: u8, title: &'a str },
    /// A code fence is a sequence of three consecutive backtick characters (`). A code block begins with a code fence, preceded by up to three spaces of indentation.
    /// The line with the opening code fence may optionally contain some text following the code fence; this is trimmed of leading and trailing spaces and called the info string.
    /// The content of the code block consists of all subsequent lines, until a closing code fence.
    /// The content of a code fence is treated as literal text, not parsed as inlines.
    CodeBlock {
        metadata: Option<&'a str>,
        content: String,
    },
    /// A link reference definition consists of a link label, followed by a colon (:), one space, link destination, and an optional link title, which if it is present must be separated from the link destination by spaces. No further character may occur.
    /// A link reference definition does not correspond to a structural element of a document. Instead, it defines a label which can be used in reference links and reference-style images elsewhere in the document. Link reference definitions can come either before or after the links that use them.
    LinkReferenceDefinition {
        link_label: &'a str,
        link_destination: &'a str,
        link_title: Option<&'a str>,
    },
    /// A sequence of non-blank lines that cannot be interpreted as other kinds of blocks forms a paragraph. The contents of the paragraph are the result of parsing the paragraph’s raw content as inlines. The paragraph’s raw content is formed by concatenating the lines and removing initial and final spaces.
    Paragraph { lines: InlineContent<'a> },
}

#[derive(Debug)]
struct InlineContent<'a> {
    blocks: Vec<InlineBlock<'a>>,
}

#[derive(Debug)]
enum InlineBlock<'a> {
    /// A code span begins with a backtick and ends with a backtick. The contents of the code span are the characters between these two backticks, normalized in the following ways:
    /// - Line endings are converted to spaces.
    /// - If the resulting string both begins and ends with a space character, but does not consist entirely of space characters, a single space character is removed from the front and back.
    CodeSpan(InlineContent<'a>),
    /// Italic text
    ///
    /// Markdown treats asterisks (*) as indicators of emphasis. Text wrapped with one * will be wrapped with an HTML <em> tag; double *’s will be wrapped with an HTML <strong> tag.
    Emphasis1(InlineContent<'a>),
    /// Bold text
    ///
    /// Markdown treats asterisks (*) as indicators of emphasis. Text wrapped with one * will be wrapped with an HTML <em> tag; double *’s will be wrapped with an HTML <strong> tag.
    Emphasis2(InlineContent<'a>),
    /// A link text consists of a sequence of one+ inline elements enclosed by square brackets ([ and ]). The following rules apply:
    /// - Links may not contain other links, at any level of nesting.
    /// - Brackets are allowed in the link text only if (a) they are backslash-escaped.
    /// - Backtick code spans, autolinks, and raw HTML tags bind more tightly than the brackets in link text. Thus, for example, [foo`]` could not be a link text, since the second ] is part of a code span.
    /// - The brackets in link text bind more tightly than markers for emphasis and strong emphasis. Thus, for example, *[foo*](url) is a link.
    ///
    /// A link destination consists of either
    /// - a sequence of zero or more characters between an opening < and a closing > that contains no line endings or unescaped < or > characters
    /// - a nonempty sequence of characters that does not start with <, does not include ASCII control characters or space character, and includes parentheses only if (a) they are backslash-escaped or (b) they are part of a balanced pair of unescaped parentheses.
    ///
    /// A link title consists of either
    /// - a sequence of zero or more characters between straight double-quote characters ("), including a " character only if it is backslash-escaped
    /// - a sequence of zero or more characters between straight single-quote characters ('), including a ' character only if it is backslash-escaped
    ///
    /// Although link titles may span multiple lines, they may not contain a blank line.
    ///
    /// An inline link consists of a link text, optionally followed immediately by a left parenthesis (link destination, an optional link title, and a right parenthesis ).
    /// These four components may be separated by spaces. If both link destination and link title are present, they must be separated by spaces.
    ///
    /// Example: [link](/uri "title")
    Link {
        text: InlineContent<'a>,
        destination: Option<&'a str>,
        title: Option<&'a str>,
    },
    /// Syntax for images is like the syntax for links, with one difference. Instead of link text, we have an image description. The rules for this are the same as for link text, except that (a) an image description starts with ![ rather than [, and (b) an image description may contain links.
    Image {
        description: InlineContent<'a>,
        destination: Option<&'a str>,
        title: Option<&'a str>,
    },
    /// Autolinks are absolute URIs and email addresses inside < and >. They are parsed as links, with the URL or email address as the link label.
    Autolink {
        link: &'a str,
    },
    Text(&'a str),
}

fn main() {
    let source = r#"
```zig
// Before:
assert(header_b != null or replica.commit_min == replica.op_checkpoint);

// After:
if (header_b == null) assert(replica.commit_min == replica.op_checkpoint);
```
"#
    .trim();
    let source2 = r#"
A short one today!

When using assertions heavily, a common pattern is asserting an implication:
```zig
assert(a implies b);
```

Most programming languages don’t have special syntax for implication, as you don’t often branch based on implications. But in my experience, you want to assert implications all the time!
Recall that **and**, **or**, and **not** logical operators form a basis, so an implication can be expressed in terms of disjunction and negation:
```
A ⇒ B ⇔ ¬A ∨ B
```

This tautology is how asserting that **a** implies **b** gets expressed by default:
```zig
assert(!a or b);
```

I find this form hard to read, and suggest using the **if** instead:
if (a) assert(b);

From a recent code change:
```zig
// Before:
assert(header_b != null or replica.commit_min == replica.op_checkpoint);

// After:
if (header_b == null) assert(replica.commit_min == replica.op_checkpoint);
```
"#.trim();
    let md = parse_markdown(&source.lines().collect::<Vec<_>>());
}

fn parse_markdown<'a>(mut lines: &[&'a str]) -> Markdown<'a> {
    let mut blocks = Vec::new();
    loop {
        if lines.is_empty() {
            todo!();
        }
        let mut line = lines[0];
        if line.starts_with("---") {
            while let Some(remainder) = line.strip_prefix('-') {
                line = remainder;
            }
            assert!(line.is_empty());
            blocks.push(MarkdownBlock::LineBreak);
            lines = &lines[1..];
        } else if line.starts_with("***") {
            while let Some(remainder) = line.strip_prefix('*') {
                line = remainder;
            }
            assert!(line.is_empty());
            blocks.push(MarkdownBlock::LineBreak);
            lines = &lines[1..];
        } else if line.starts_with("___") {
            while let Some(remainder) = line.strip_prefix('_') {
                line = remainder;
            }
            assert!(line.is_empty());
            blocks.push(MarkdownBlock::LineBreak);
            lines = &lines[1..];
        } else if line.starts_with('#') {
            let mut level = 0;
            while let Some(remainder) = line.strip_prefix('#') {
                level += 1;
                line = remainder;
            }
            assert!(level <= 6);
            let title = line.strip_prefix(' ').unwrap().trim();
            assert!(!title.trim().is_empty());
            blocks.push(MarkdownBlock::Heading { level, title });
            lines = &lines[1..];
        } else if line.starts_with("```") {
            let metadata = line.strip_prefix("```").unwrap().trim();
            let metadata = if metadata.is_empty() {
                None
            } else {
                Some(metadata)
            };
            let mut content_lines_count = 0;
            while !lines[content_lines_count + 1].starts_with("```") {
                content_lines_count += 1;
            }
            let content_lines = &lines[1..=content_lines_count];
            assert!(
                lines[content_lines_count + 1]
                    .strip_prefix("```")
                    .unwrap()
                    .is_empty()
            );
            blocks.push(MarkdownBlock::CodeBlock {
                metadata,
                content: content_lines.join("\n"),
            });
            lines = &lines[content_lines_count + 2..];
        } else if line.starts_with('[') {
            todo!("parse link reference definition");
        } else if line.starts_with('>') {
            let mut quote_lines = Vec::new();
            while let Some(line) = lines[quote_lines.len()].strip_prefix("> ") {
                quote_lines.push(line)
            }
            assert!(!quote_lines.is_empty());
            let content = parse_markdown(&quote_lines);
            blocks.push(MarkdownBlock::BlockQuote { content });
            lines = &lines[quote_lines.len()..];
        } else if line.starts_with('-') {
            // FIXME: '*' and '+' should also work
            let mut items = Vec::new();
            while let Some(item_start) = lines[1].strip_prefix('-') {
                let item_start = item_start.strip_prefix(' ').unwrap();
                let mut item_lines = Vec::new();
                item_lines.push(item_start);
                while let Some(item_line) = lines[item_lines.len() + 1].strip_prefix("  ") {
                    item_lines.push(item_line);
                }
                lines = &lines[item_lines.len()..];
                items.push(parse_markdown(&item_lines));
            }
            assert!(!items.is_empty());
            blocks.push(MarkdownBlock::BulletList { items });
        }
        dbg!(&blocks);
        todo!()
    }
}
