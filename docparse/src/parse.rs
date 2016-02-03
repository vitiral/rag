
extern crate regex;

use std;
use regex::Regex;
use types;
use comments;

/// find the match for the next opening bracket
fn find_match(text: &str, open: char, close: char) -> types::Result<usize> {
    let mut num_opens: usize = 1;
    let mut chars = text.chars();
    let mut i = 0;
    loop {
        let c = match chars.next() {
            Some(c) => c,
            None => return Err(types::ParseError::Empty),
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
    let white = comments::whiteout(comment).unwrap();
    assert_eq!(find_match(&white, '{', '}').unwrap(), comment.len());
    let block = "/*comment{*/}";
    let white = comments::whiteout(block).unwrap();
    assert_eq!(find_match(&white, '{', '}').unwrap(), block.len());
}

/// parse the captured text into a CodeBlock
fn get_block<'a>(text: &'a str, // the unaltered text with a long lifetime
                      white: &str, // the whiteout-ed text for parsing
                      cap: &regex::Captures,
                      block_delim: (char, char))
                      -> types::Result<types::CodeBlock<'a>> {
    // we can use unwrap here because there is no possible way that the pattern doesn't exist
    // in addition, we can use unicode slices because we know (from the regexp) that
    // the char boundaries are always valid
    let kind = match cap.at(2).unwrap() {
        "fn" => types::CodeTypes::Fn,
        "struct" => types::CodeTypes::Struct,
        "enum" => types::CodeTypes::Enum,
        "trait" => types::CodeTypes::Trait,
        "mod" => types::CodeTypes::Mod,
        _ => unreachable!(),
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
    Ok(types::CodeBlock {
        name: name,
        kind: kind,
        sig: sig,
        pos: pos,
    })
}


/// get a regular expression for searching for the specified block
fn block_regex(kind: &str, open: &str) -> Result<Regex, regex::Error> {
    // must start at a newline and can have some whitespace before it
    let pat = format!(r"(^|\n)\s*?({})\s+(<.*>)?\s*(\w+).*?{}", kind, open);
    Regex::new(&pat)
}

/// parse the given block of text for all rust code signatures
/// and their positions
pub fn get_code_blocks(text: &str) -> types::Result<Vec<types::CodeBlock>> {
    // FIXME: the compiled regexps need to be made into global variables
    let white = try!(comments::whiteout(text));
    // println!(">>>>>> WHITE\n{}>>>>>", white);
    let mut out: Vec<types::CodeBlock> = vec![];
    let docpat = block_regex("fn|struct|enum|trait|mod", r"\{").unwrap();
    for cap in docpat.captures_iter(&white) {
        let v = try!(get_block(text, &white, &cap, ('{', '}')));
        out.push(v);
    }
    let tuplepat = block_regex("struct", r"\(").unwrap();
    for cap in tuplepat.captures_iter(&white) {
        out.push(try!(get_block(text, &white, &cap, ('(', ')'))));
    }
    let nullpat = block_regex("struct", ";").unwrap();
    for cap in nullpat.captures_iter(&white) {
        out.push(try!(get_block(text, &white, &cap, (0 as char, 0 as char))));
    }
    // sort by start position
    out.sort_by(|a, b| a.pos.0.cmp(&b.pos.0));
    Ok(out)
}

static TEST_TEXT: &'static str = "
//! file level documentation
//! more file documentation

/// \
                                  documentation for my function
/// some more docs
fn myfun(x: \
                                  i32, y:i64) -> u32 {
    // here are some comments inside the \
                                  function
    let x = 4;
    if x == 7 {
        println!(\"what \
                                  the heck is happening?\");
    }
    {{{}}} // just to cause \
                                  problems
    // end myfun
}

some stuff after the function

/// \
                                  documentation for myenum
enum myenum {
    x,
    y,
    z,
    \
                                  // end myenum
}

/// documentation for std struct
struct \
                                  mystruct {
    x: i32,
    y: f32,
    // end mystruct
}

/// \
                                  documentation for tuple struct
struct tuplestruct(u32, f64
    \
                                  // end tuplestruct
)

/// documentation for nullstruct
struct \
                                  nullstruct /*some terrible documentation;*/ /*end nullstruct*/ \
                                  ;

/// documentation for mymod
/// some more mod documentation
\
                                  mod mymod {
    fn myfun2(x: i32, y: f64) -> f64 {
        // \
                                  here are some comments inside myfun
        if x == 7 {
            \
                                  println!(\"what the heck is x\");
        }
        // a \
                                  comment
        /* terrible block comment }*/
        // \
                                  terrible } line comment
        // end myfun2
    }
    // some \
                                  more comments
    // end mymod
}

some stuff after mod

";

#[test]
fn test_get_code_blocks() {
    let blocks = get_code_blocks(TEST_TEXT).unwrap();
    let mut n = 0;

    let block = &blocks[n];
    assert_eq!(block.name, "myfun");
    match block.kind {
        types::CodeTypes::Fn => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "fn myfun(x: i32, y:i64) -> u32");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("fn myfun"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myfun\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "myenum");
    match block.kind {
        types::CodeTypes::Enum => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "enum myenum");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("enum myenum"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myenum\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "mystruct");
    match block.kind {
        types::CodeTypes::Struct => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct mystruct");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct mystruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end mystruct\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "tuplestruct");
    match block.kind {
        types::CodeTypes::Struct => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct tuplestruct");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct tuplestruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end tuplestruct\n)"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "nullstruct");
    match block.kind {
        types::CodeTypes::Struct => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig,
               "struct nullstruct /*some terrible documentation;*/ /*end nullstruct*/");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct nullstruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" /*end nullstruct*/ ;"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "mymod");
    match block.kind {
        types::CodeTypes::Mod => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "mod mymod");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("mod mymod"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end mymod\n}"));
}
