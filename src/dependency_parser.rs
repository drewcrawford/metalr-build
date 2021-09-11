//! This parses a dependency file.  We use this to emit cargo:rerun-if-changed=PATH

use std::path::{PathBuf};
use std::io::Read;

pub fn tell_cargo_about_dependencies(dependency_file: PathBuf) {
    let mut file = std::fs::File::open(dependency_file).unwrap();
    let mut str = String::new();
    let _ = file.read_to_string(&mut str).unwrap();
    for dependency in parse(&str) {
        println!("cargo:rerun-if-changed='{}'",dependency);
    }
}

// Accepts input like `whatever: file.metal file.h` supporting newlines and escapes
fn parse(parse_me: &str) -> Vec<String> {
    let mut iter = parse_me.chars();
    //first we advance until we find a `:`
    let _ = iter.by_ref().take_while(|p| p != &':').for_each(drop);
    //drop whitespace
    let _ = iter.by_ref().take_while(|p| p != &' ').for_each(drop);

    let mut out = Vec::new();
    //if we immediately parsed `\` in the previoius iteration
    let mut escaped = false;
    let mut current_str = String::new();
    for char in iter {
        //println!("match {}",char);
        match char {
            ' ' => {
                if current_str.len() == 0 {
                    //waiting for next string to start
                }
                else if escaped {
                    current_str.push(char);
                    escaped = false;
                }
                else {
                    out.push(current_str);
                    current_str = String::new();
                }
            }
            '\\' => {
                escaped = true;
            }
            '\n' => {
                escaped = false;
            }
            other => {
                assert!(!escaped);
                current_str.push(other);
            }
        }
    }
    out.push(current_str);
    out
}

#[test] fn test_parse() {
    let txt = r#"depedencies: /Users/drew/Code/winspike/metal-build/tests/test.metal \
  /Users/drew/Code/winspike/metal-build/tests/example1.h \
  /Users/drew/Code/winspike/metal-build/tests/example\ 2.h"#;
    let deps = parse(txt);
    assert_eq!(deps[0], "/Users/drew/Code/winspike/metal-build/tests/test.metal");
    assert_eq!(deps[1], "/Users/drew/Code/winspike/metal-build/tests/example1.h");
    assert_eq!(deps[2], "/Users/drew/Code/winspike/metal-build/tests/example 2.h");
    assert_eq!(deps.len(), 3);
}