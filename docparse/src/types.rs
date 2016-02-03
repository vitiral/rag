use std;

#[derive(Debug)]
pub enum ParseError {
    Empty,
}


pub type Result<T> = std::result::Result<T, ParseError>;

/// CodeTypes: types that a 
#[derive(Debug)]
pub enum CodeTypes {
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
    pub name: &'a str,    // the name of the code block, i.e. `fn myfunc` has name "myfunc"
    pub kind: CodeTypes,  // the type of the block, i.e. Fn, Struct, etc.
    pub sig: &'a str,     // the sig, i.e. "fn myfunc(x: i32) -> i32"
    pub pos: (usize, usize),  // the character positions of the block inside the file
}

// #[derive(Debug)]
// pub struct Doc {
//     // path: &'static str, // FIXME: this should be a path object of some kind
//     name: &'static str,
//     kind: CodeTypes,
//     sig: &'static str,
//     doc: &'static str,
// }
