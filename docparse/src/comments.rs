///! mod for dealing with comments in various ways

use types;

/// whiteout comments by inserting ' ' for each **byte** that a comment
/// would have taken up before.
/// This function intenionally preserves the size of the string
/// so that text.len() == white.len()
pub fn whiteout(text: &str) -> types::Result<String> {
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
fn test_whiteout() {
    let input = "something // with a comment\n";
    let expect = "something                  \n";
    assert_eq!(whiteout(input).unwrap(), expect);

    let input = "something /*comment \n in*/ middle\n";
    let expect = "something           \n      middle\n";
    assert_eq!(whiteout(input).unwrap(), expect);
}

