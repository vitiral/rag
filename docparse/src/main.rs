extern crate regex;

use regex::Regex;

#[derive(Debug)]
enum ParseError {
    Empty,
    InvalidComment,
}

type Result<T> = std::result::Result<T, ParseError>;

/// CodeTypes: types that a 
#[derive(Debug)]
enum CodeTypes {
    Fn,
    Enum,
    Struct,
    Trait,
    Mod,
}

/// CodeBlock: the unit of documentation
///
/// contains a single unit of documentation related o
/// one of the rust docmented types
#[derive(Debug)]
pub struct CodeBlock <'a>{
    name: &'a str,
    kind: CodeTypes,
    sig: &'a str,
    pos: (usize, usize),
}

// #[derive(Debug)]
// pub struct Doc {
//     // path: &'static str, // FIXME: this should be a path object of some kind
//     name: &'static str,
//     kind: CodeTypes,
//     sig: &'static str,
//     doc: &'static str,
// }

/// find the match for the next opening bracket
fn find_match(text: &str, open: char, close: char) -> Result<usize> {
    let mut num_opens: usize = 1;
    let mut chars = text.chars();
    let mut i = 0;
    loop {
        let c = match chars.next() {
            Some(c) => c,
            None => return Err(ParseError::Empty),
        };
        i += 1;
        if c == open {
            num_opens += 1;
        } else if c == close {
            num_opens -= 1;
            if num_opens == 0 {
                return Ok(i);
            }
        }
    }
}

#[test]
fn test_find_match() {
    assert_eq!(find_match("12345}", '{', '}').unwrap(), 6);
    assert_eq!(find_match(&"  {12345}"[3..], '{', '}').unwrap(), 6);
    let comment = "//badcomment {\n}";
    let white = whiteout_comments(comment).unwrap();
    assert_eq!(find_match(&white, '{', '}').unwrap(), comment.len());
    let block = "/*comment{*/}";
    let white = whiteout_comments(block).unwrap();
    assert_eq!(find_match(&white, '{', '}').unwrap(), block.len());
}

/// parse the captured text into a CodeBlock
fn parse_captured<'a>(text: &'a str,
                  cap: &regex::Captures,
                  block_delim: (char, char)) -> Result<CodeBlock<'a>> {
    // we can use unwrap here because there is no possible way that the pattern doesn't exist
    // in addition, we can use unicode slices because we know (from the regexp) that
    // the char boundaries are always valid
    let kind = match cap.at(2).unwrap() {
        "fn"     => CodeTypes::Fn,
        "struct" => CodeTypes::Struct,
        "enum"   => CodeTypes::Enum,
        "trait"  => CodeTypes::Trait,
        "mod"    => CodeTypes::Mod,
        _        => unreachable!(),
    };
    // pos starts at declaration and goes until closing brace
    let mut pos = cap.pos(0).unwrap();
    let sigpos = pos.clone();
    pos.0 = cap.pos(2).unwrap().0;
    if block_delim.0 as u8 != 0 {
        pos.1 = pos.1 + find_match(&(text[pos.1..]), block_delim.0, block_delim.1).unwrap();
    }
    // get rid of the close in sig and trim it
    let namepos = cap.pos(4).unwrap();
    let sig = &text[pos.0..sigpos.1];
    let sig = (&sig[0..sig.len() - 1]).trim();
    let name = &text[namepos.0..namepos.1];
    Ok(CodeBlock {
        name: name,
        kind: kind,
        sig: sig,
        pos: pos,
    })
}

/// whiteout comments by inserting ' ' for each **byte** that a comment
/// would have taken up before.
/// This function intenionally preserves the size of the string
/// so that text.len() == white.len()
fn whiteout_comments(text: &str) -> Result<String> {
    let mut white = String::with_capacity(text.len());
    let mut chars = text.chars();
    'outer: loop {
        match chars.next() {
            Some('/') => {
                // it could be a comment
                match chars.next() {
                    Some('*') => {
                        // it is a block comment, whitespace until end of block
                        for _ in 0..2 { white.push(' ') };
                        let mut last = 0 as char;
                        for c in chars.by_ref() {
                            if c == '\n' {
                                white.push(c);  // we preserve newlines
                            } else {
                                for _ in 0..c.len_utf8(){ white.push(' ') };
                            }
                            if last == '*' && c == '/' {
                                break;
                            }
                            last = c;
                        }
                    },
                    Some('/') => {
                        // it is one of the line comments, whiteout until newline
                        for _ in 0..2 { white.push(' ') };
                        for c in chars.by_ref() {
                            if c == '\n' {
                                white.push(c);
                                break;
                            }
                            else {
                                for _ in 0..c.len_utf8(){ white.push(' ') };
                            }
                        }
                    },
                    Some(c) => {
                        // any other character
                        white.push('/');
                        white.push(c);
                    }
                    None => {
                        // file ended with '/'
                        white.push('/');
                        break 'outer;
                    }
                }
            },
            Some(c) => white.push(c),
            None => break,
        }
    }
    assert_eq!(text.len(), white.len());
    Ok(white)
}

#[test]
fn test_whiteout_comments() {
    let input  = "something // with a comment\n";
    let expect = "something                  \n";
    assert_eq!(whiteout_comments(input).unwrap(), expect);

    let input  = "something /*comment \n in*/ middle\n";
    let expect = "something           \n      middle\n";
    assert_eq!(whiteout_comments(input).unwrap(), expect);
}

/// parse the given block of text for all rust code signatures
/// and their positions
fn parse_blocks(text: &str) -> Result<Vec<CodeBlock>> {
    let white = try!(whiteout_comments(text));
    // FIXME: this should be made a global variable somehow
    let mut out: Vec<CodeBlock> = vec!();
    let docpat = Regex::new(r"(^|\n)\s*?(fn|struct|enum|trait|mod)\s+(<.*>)?\s*(\w+).*?\{").unwrap();
    for cap in docpat.captures_iter(&white) {
        out.push(try!(parse_captured(text, &cap, ('{', '}'))));
    }
    let tuplepat = Regex::new(r"(^|\n)\s*?(struct)\s+(<.*>)?\s*(\w+).*?\(").unwrap();
    for cap in tuplepat.captures_iter(text) {
        out.push(try!(parse_captured(text, &cap, ('(', ')'))));
    }
    let nullpat = Regex::new(r"(^|\n)\s*?(struct)\s+(<.*>)?\s*(\w+).*?\(").unwrap();
    for cap in nullpat.captures_iter(text) {
        out.push(try!(parse_captured(text, &cap, (0 as char, 0 as char))));
    }
    // sort by start position
    out.sort_by(|a, b| a.pos.0.cmp(&b.pos.0));
    Ok(out)
}

static TEST_TEXT: &'static str = "
//! file level documentation
//! more file documentation

/// documentation for my function
/// some more docs
fn myfun(x: i32, y:i64) -> u32 {
    // here are some comments inside the function
    ...
    // end myfun
}

some stuff after the function

/// documentation for myenum
enum myenum {
    x,
    y,
    z,
    // end myenum
}

/// documentation for std struct
struct mystruct {
    x: i32,
    y: f32,
    // end mystruct
}

/// documentation for tuple struct
struct tuplestruct(u32, f64
    // end tuplestruct
)

/// documentation for nullstruct
struct nullstruct /*some terrible documentation;*/ /*end of nullstruct*/;

/// documentation for mymod
/// some more mod documentation
mod mymod {
    fn myfun2(x: i32, y: f64) -> f64 {
        // here are some comments inside myfun
        ...
        /* terrible block comment }*/
        // end myfun2
    }
    // end mymod
}

some stuff after mod
";

#[test]
fn test_parse_blocks() {
    let mut n = 0;
    let blocks = parse_blocks(TEST_TEXT).unwrap();
    let block = &blocks[n];
    assert_eq!(block.name, "myfun");
    match block.kind {
        CodeTypes::Fn => {},
        _ => assert!(false),
    }
    assert_eq!(block.sig, "fn myfun(x: i32, y:i64) -> u32");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("fn myfun"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myfun\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "myenum");
    match block.kind {
        CodeTypes::Enum => {},
        _ => assert!(false),
    }
    assert_eq!(block.sig, "enum myenum");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("enum myenum"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myenum\n}"));

    // n += 1;
    // let block = &blocks[n];
    // assert_eq!(block.name, "mystruct");
    // match block.kind {
    //     CodeTypes::Enum => {},
    //     _ => assert!(false),
    // }
    // assert_eq!(block.sig, "enum myenum");
    // assert!(&TEST_TEXT[block.pos.0..].starts_with("enum myenum"));
    // assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myenum\n}"));
}


fn main() {
    let myfun = "
/// documentation for my function
/// some more docs
fn myfun(x: i32, \
                y:i64) -> u32 {
    ...
}";
    println!("{:?}", parse_blocks(myfun));
}
