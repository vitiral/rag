extern crate regex;
use regex::Regex;

pub mod types;
mod comments;
mod parse;


fn main() {
    let myfun = "
/// documentation for my function
/// some more docs
fn myfun(x: i32, y:i64) -> \
                 u32 {
    ...
}";
    println!("{:?}", parse::get_code_blocks(myfun));
}
