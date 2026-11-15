//! Invalid exact-build selections must fail before opening a window or socket.

use std::process::Command;

#[test]
fn invalid_receive_build_selection_exits_before_starting_a_client() {
    for args in [
        vec!["--receive-build"],
        vec!["--receive-build", "v22"],
        vec!["--version", "2.05", "--receive-build", "v22"],
        vec!["--version", "2.1", "--receive-build", "GNW31101"],
        vec!["--version", "2.1", "--receive-build", "2.1.0.4"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_seihai-mae"))
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("receive"),
            "{args:?}: {:?}",
            output.stderr
        );
    }
}
