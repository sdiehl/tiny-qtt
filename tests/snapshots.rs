use std::fs;
use tiny_qtt::check_str;

#[test]
fn cases() {
    insta::glob!("cases/*.qtt", |path| {
        let src = fs::read_to_string(path).unwrap();
        let out = match check_str(&src) {
            Ok(s) => s,
            Err(e) => format!("ERROR: {e}\n"),
        };
        insta::with_settings!({ snapshot_path => "snapshots", prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(out);
        });
    });
}
