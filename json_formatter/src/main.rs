use std::io::{BufReader, BufWriter, Read, Write, stdin, stdout};

fn main() -> anyhow::Result<()> {
    let mut stdin = BufReader::new(stdin()).bytes().peekable();
    let mut stdout = BufWriter::new(stdout());

    let mut ctx = ParsingContext::default();
    while let Some(ch) = stdin.next() {
        let ch = ch?;
        match ch {
            b'[' => {
                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "[")?;
                ctx.parents.push(ParentTy::Array);
                ctx.indentation += 1;
                ctx.has_prev = false;
                ctx.state = ParsingState::InArray;
            }
            b'{' => {
                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "{{")?;
                ctx.parents.push(ParentTy::Object);
                ctx.indentation += 1;
                ctx.has_prev = false;
                ctx.state = ParsingState::InStructWithoutName;
            }
            b']' => {
                assert_eq!(ctx.parents.pop(), Some(ParentTy::Array));
                ctx.indentation -= 1;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "]")?;
                ctx.state = match ctx.parents.last() {
                    Some(ParentTy::Array) => ParsingState::InArray,
                    Some(ParentTy::Object) => ParsingState::InStructWithoutName,
                    None => ParsingState::TopLevel,
                };

                ctx.has_prev = true;
                let _ = stdin.next_if(|x| match x {
                    Ok(ch) => *ch == b',',
                    Err(_) => false,
                });
            }
            b'}' => {
                assert_eq!(ctx.parents.pop(), Some(ParentTy::Object));
                ctx.indentation -= 1;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "}}")?;
                ctx.state = match ctx.parents.last() {
                    Some(ParentTy::Array) => ParsingState::InArray,
                    Some(ParentTy::Object) => ParsingState::InStructWithoutName,
                    None => ParsingState::TopLevel,
                };

                ctx.has_prev = true;
                let _ = stdin.next_if(|x| match x {
                    Ok(ch) => *ch == b',',
                    Err(_) => false,
                });
            }
            b'"' => {
                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "\"")?;
                let found = consume_string_to_end(&mut stdin, &mut stdout)?;
                assert!(found);

                if let ParsingState::InStructWithoutName = ctx.state {
                    ctx.state = ParsingState::InStructWithName;

                    assert_eq!(stdin.next().transpose()?, Some(b':'));
                    write!(stdout, ": ")?;
                } else {
                    if let ParsingState::InStructWithName = ctx.state {
                        ctx.state = ParsingState::InStructWithoutName
                    }

                    ctx.has_prev = true;
                    let _ = stdin.next_if(|x| match x {
                        Ok(ch) => *ch == b',',
                        Err(_) => false,
                    });
                }
            }
            digit @ (b'-' | b'0'..=b'9') => {
                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "{}", digit as char)?;
                while let Some(ch) = peek(&mut stdin)
                    && ch.is_ascii_digit()
                {
                    let ch = stdin.next().unwrap()?;
                    write!(stdout, "{}", ch as char)?;
                }
                if let Some(ch) = peek(&mut stdin)
                    && ch == b'.'
                {
                    let ch = stdin.next().unwrap()?;
                    write!(stdout, "{}", ch as char)?;
                    while let Some(ch) = peek(&mut stdin)
                        && ch.is_ascii_digit()
                    {
                        let ch = stdin.next().unwrap()?;
                        write!(stdout, "{}", ch as char)?;
                    }
                }

                if let ParsingState::InStructWithName = ctx.state {
                    ctx.state = ParsingState::InStructWithoutName
                }
                ctx.has_prev = true;
                let _ = stdin.next_if(|x| match x {
                    Ok(ch) => *ch == b',',
                    Err(_) => false,
                });
            }
            b'n' => {
                assert_eq!(stdin.next().transpose()?.unwrap(), b'u');
                assert_eq!(stdin.next().transpose()?.unwrap(), b'l');
                assert_eq!(stdin.next().transpose()?.unwrap(), b'l');

                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "null")?;

                if let ParsingState::InStructWithName = ctx.state {
                    ctx.state = ParsingState::InStructWithoutName
                }
                ctx.has_prev = true;
                let _ = stdin.next_if(|x| match x {
                    Ok(ch) => *ch == b',',
                    Err(_) => false,
                });
            }
            b't' => {
                assert_eq!(stdin.next().transpose()?.unwrap(), b'r');
                assert_eq!(stdin.next().transpose()?.unwrap(), b'u');
                assert_eq!(stdin.next().transpose()?.unwrap(), b'e');

                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "true")?;

                if let ParsingState::InStructWithName = ctx.state {
                    ctx.state = ParsingState::InStructWithoutName
                }
                ctx.has_prev = true;
                let _ = stdin.next_if(|x| match x {
                    Ok(ch) => *ch == b',',
                    Err(_) => false,
                });
            }
            b'f' => {
                assert_eq!(stdin.next().transpose()?.unwrap(), b'a');
                assert_eq!(stdin.next().transpose()?.unwrap(), b'l');
                assert_eq!(stdin.next().transpose()?.unwrap(), b's');
                assert_eq!(stdin.next().transpose()?.unwrap(), b'e');

                add_comma(&ctx, &mut stdout)?;
                indent(&ctx, &mut stdout)?;
                write!(stdout, "false")?;

                if let ParsingState::InStructWithName = ctx.state {
                    ctx.state = ParsingState::InStructWithoutName
                }
                ctx.has_prev = true;
                let _ = stdin.next_if(|x| match x {
                    Ok(ch) => *ch == b',',
                    Err(_) => false,
                });
            }
            c => {
                todo!("unknown character {}", c as char)
            }
        }
        while let Some(ch) = peek(&mut stdin)
            && ch.is_ascii_whitespace()
        {
            let _ = stdin.next();
        }
    }
    writeln!(stdout)?;

    stdout.flush()?;

    Ok(())
}

fn indent(ctx: &ParsingContext, mut output: impl Write) -> std::io::Result<()> {
    if ctx.state != ParsingState::InStructWithName && ctx.state != ParsingState::TopLevel {
        writeln!(output)?;
        write!(output, "{}", "  ".repeat(ctx.indentation))?;
    }
    Ok(())
}
fn add_comma(ctx: &ParsingContext, mut output: impl Write) -> std::io::Result<()> {
    if ctx.state != ParsingState::InStructWithName && ctx.has_prev {
        write!(output, ",")?;
    }
    Ok(())
}

fn peek(
    it: &mut std::iter::Peekable<impl Iterator<Item = Result<u8, std::io::Error>>>,
) -> Option<u8> {
    it.peek().and_then(|x| match x {
        Ok(ch) => Some(*ch),
        Err(_err) => None,
    })
}

#[derive(Debug, PartialEq, Eq)]
enum ParentTy {
    Array,
    Object,
}

#[derive(Debug, Default, PartialEq)]
enum ParsingState {
    InStructWithName,
    InStructWithoutName,
    InArray,
    #[default]
    TopLevel,
}

#[derive(Default)]
struct ParsingContext {
    state: ParsingState,
    parents: Vec<ParentTy>,
    indentation: usize,
    has_prev: bool,
}

/// Gets escaped string without opening quote and returns bytes count to closing quote(including it)
/// Returns false if closing quote not found
fn consume_string_to_end(
    input: &mut impl Iterator<Item = Result<u8, std::io::Error>>,
    mut output: impl Write,
) -> std::io::Result<bool> {
    let mut last_backslash = false;
    for byte in input {
        let byte = byte?;
        output.write_all(&[byte])?;
        match byte {
            b'\\' if !last_backslash => {
                last_backslash = true;
            }
            b'"' if !last_backslash => {
                return Ok(true);
            }
            _ => last_backslash = false,
        }
    }
    Ok(false)
}
