extern crate docparse;

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
    let blocks = docparse::parse::get_code_blocks(TEST_TEXT).unwrap();
    let mut n = 0;

    let block = &blocks[n];
    assert_eq!(block.name, "myfun");
    match block.kind {
        docparse::types::CodeTypes::Fn => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "fn myfun(x: i32, y:i64) -> u32");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("fn myfun"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myfun\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "myenum");
    match block.kind {
        docparse::types::CodeTypes::Enum => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "enum myenum");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("enum myenum"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end myenum\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "mystruct");
    match block.kind {
        docparse::types::CodeTypes::Struct => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct mystruct");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct mystruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end mystruct\n}"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "tuplestruct");
    match block.kind {
        docparse::types::CodeTypes::Struct => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "struct tuplestruct");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("struct tuplestruct"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end tuplestruct\n)"));

    n += 1;
    let block = &blocks[n];
    assert_eq!(block.name, "nullstruct");
    match block.kind {
        docparse::types::CodeTypes::Struct => {}
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
        docparse::types::CodeTypes::Mod => {}
        _ => assert!(false),
    }
    assert_eq!(block.sig, "mod mymod");
    assert!(&TEST_TEXT[block.pos.0..].starts_with("mod mymod"));
    assert!(&TEST_TEXT[..block.pos.1].ends_with(" // end mymod\n}"));
}
