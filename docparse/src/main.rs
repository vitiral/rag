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

fn play_re() {
    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    let text = "2012-03-14, 2013-01-01 and 2014-07-05";
    for cap in re.captures_iter(text) {
        println!("Month: {} Day: {} Year: {}",
                cap.at(2).unwrap_or(""), cap.at(3).unwrap_or(""),
                cap.at(1).unwrap_or(""));
    }
}

/// increment chars until the comment is done. Assumes the char before
/// was '/'
fn ignore_comment(chars: &mut std::str::Chars) -> Result<(usize, char)> {
    let c = match chars.next() {
        Some(c) => c,
        None => return Ok((0, 0 as char)),
    };
    let mut i: usize = 1;  // already used `next` once
    match c {
        '*' => {
            // block comment, ignore until end of block
            let mut last = 0 as char;
            for c in chars {
                i += 1;
                if last == '*' && c == '/' {
                    return Ok((i, c));
                }
                last = c;
            } // TODO: no more characters... panic? (invalid comment)
            Err(ParseError::InvalidComment)
        },
        '/' => {
            // one of the regular comments, ignore until newline
            for c in chars {
                i += 1;
                if c == '\n' {
                    return Ok((i, c));
                }
            }
            Ok((i, c))
        },
        _ => Ok((i, c)),
    }
}

/// find the match for the next opening bracket
fn find_match(text: &str, open: char, close: char) -> Result<usize> {
    let mut num_opens: usize = 1;
    let mut chars = text.chars();
    let mut i = 0;
    loop {
        let mut c = match chars.next() {
            Some(c) => c,
            None => return Err(ParseError::Empty),
        };
        i += 1;
        if c == '/' {
            // it might be a comment, if it is, ignore everything until the comment ends
            let (plus, _c) = try!(ignore_comment(&mut chars));
            i += plus;
            c = _c;
        }
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
    assert_eq!(find_match(comment, '{', '}').unwrap(), comment.len());
    let block = "/*comment{*/}";
    assert_eq!(find_match(block, '{', '}').unwrap(), block.len());
}

fn parse_captured<'a>(text: &str,
                  cap: &regex::Captures<'a>,
                  block_delim: (char, char)) -> Result<CodeBlock<'a>> {
    // we can use unwrap here because there is no possible way that the pattern doesn't exist
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
    pos.0 = cap.pos(2).unwrap().0;
    if block_delim.0 as u8 != 0 {
        pos.1 = pos.1 + find_match(&(text[pos.1..]), block_delim.0, block_delim.1).unwrap();
    }
    // get rid of the close in sig and trim it
    let sig = cap.at(0).unwrap();
    let sig = (&sig[0..sig.len() - 1]).trim();
    let name = cap.at(4).unwrap();
    Ok(CodeBlock {
        name: name,
        kind: kind,
        sig: sig,
        pos: pos,
    })
}

/// parse the given block of text for all rust code signatures
/// and their positions
fn parse_blocks(text: &str) -> Result<Vec<CodeBlock>> {
    // FIXME: this should be made a global variable somehow
    let mut out: Vec<CodeBlock> = vec!();
    let docpat = Regex::new(r"(^|\n)\s*?(fn|struct|enum|trait|mod)\s+(<.*>)?\s*(\w+).*?\{").unwrap();
    for cap in docpat.captures_iter(text) {
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
    println!("my fun:\n{}", myfun);
    play_re();
    println!("{:?}", parse_blocks(myfun));
}
