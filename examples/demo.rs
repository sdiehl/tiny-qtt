use std::process;

use tiny_qtt::check_str;

const PRELUDE: &str = include_str!("prelude.qtt");
const DEMO: &str = include_str!("demo.qtt");

fn main() {
    let src = format!("{PRELUDE}\n{DEMO}");
    match check_str(&src) {
        Ok(out) => print!("{out}"),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}
