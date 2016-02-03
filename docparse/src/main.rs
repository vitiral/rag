extern crate regex;

use regex::Regex;

#[derive(Debug)]
enum ParseError {
    Empty,
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
    name: &'a str,    // the name of the code block, i.e. `fn myfunc` has name "myfunc"
    kind: CodeTypes,  // the type of the block, i.e. Fn, Struct, etc.
    sig: &'a str,     // the sig, i.e. "fn myfunc(x: i32) -> i32"
    pos: (usize, usize),  // the character positions of the block inside the file
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
fn parse_captured<'a>(text: &'a str,  // the unaltered text with a long lifetime
                      white: &str,    // the whiteout-ed text for parsing
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
    let mut pos = cap.pos(0).unwrap();
    let sigpos = pos.clone();
    pos.0 = cap.pos(2).unwrap().0;
    if block_delim.0 as u8 != 0 {
        pos.1 = pos.1 + find_match(&(white[pos.1..]), block_delim.0, block_delim.1).unwrap();
    }
    // convert from the regexp in white to actual text in text
    // also gives us a longer lifetime
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


/// get a regular expression for searching for the specified block
fn block_regex(kind: &str, open: &str) -> std::result::Result<Regex, regex::Error> {
    // must start at a newline and can have some whitespace before it
    let pat = format!(r"(^|\n)\s*?({})\s+(<.*>)?\s*(\w+).*?{}",
                      kind, open);
    Regex::new(&pat)
}

/// parse the given block of text for all rust code signatures
/// and their positions
fn parse_blocks(text: &str) -> Result<Vec<CodeBlock>> {
    // FIXME: the compiled regexps need to be made into global variables
    let white = try!(whiteout_comments(text));
    // println!(">>>>>> WHITE\n{}>>>>>", white);
    let mut out: Vec<CodeBlock> = vec!();
    let docpat = block_regex("fn|struct|enum|trait|mod", r"\{").unwrap();
    for cap in docpat.captures_iter(&white) {
        let v = try!(parse_captured(text, &white, &cap, ('{', '}')));
        out.push(v);
    }
    let tuplepat = block_regex("struct", r"\(").unwrap();
    for cap in tuplepat.captures_iter(&white) {
        out.push(try!(parse_captured(text, &white, &cap, ('(', ')'))));
    }
    let nullpat = block_regex("struct", ";").unwrap();
    for cap in nullpat.captures_iter(&white) {
        out.push(try!(parse_captured(text, &white, &cap, (0 as char, 0 as char))));
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
    let x = 4;
    if x == 7 {
        println!(\"what the heck is happening?\");
    }
    {{{}}} // just to cause problems
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
struct nullstruct /*some terrible documentation;*/ /*end nullstruct*/ ;

/// documentation for mymod
/// some more mod documentation
mod mymod {
    fn myfun2(x: i32, y: f64) -> f64 {
        // here are some comments inside myfun
        if x == 7 {
            println!(\"what the heck is x\");
        }
        // a comment
        /* terrible block comment }*/
        // terrible } line comment
        // end myfun2
    }
    // some more comments
    // end mymod
}

some stuff after mod

";

#[test]
fn test_parse_blocks() {
    let blocks = parse_blocks(TEST_TEXT).unwrap();
    let mut n = 0;

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

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "mystruct");
    match block.kind {
        CodeTypes::Struct => {},
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct mystruct");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct mystruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end mystruct\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "tuplestruct");
    match block.kind {
        CodeTypes::Struct => {},
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct tuplestruct");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct tuplestruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end tuplestruct\n)"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "nullstruct");
    match block.kind {
        CodeTypes::Struct => {},
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct nullstruct /*some terrible documentation;*/ /*end nullstruct*/");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct nullstruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" /*end nullstruct*/ ;"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "mymod");
    match block.kind {
        CodeTypes::Mod => {},
        _ => assert!(false),
    }
    assert_eq!(block.sig, "mod mymod");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("mod mymod"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end mymod\n}"));


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
