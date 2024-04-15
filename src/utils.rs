use std::io::{stdin, stdout, Write};

pub fn confirm(tip: &str, default: bool) -> bool {
    let end = if default { " [Y/n]: " } else { " [y/N]: " };
    print!("{tip} {end}");
    stdout().flush().expect("can't flush stdout");
    let mut ans = String::new();
    stdin().read_line(&mut ans).expect("can't read line");
    // if default is true, all answer that is not equals to n will be true.
    if default {
        ans.trim().to_lowercase() != "n"
    } else {
        ans.trim().to_lowercase() != "y"
    }
}
