//! Invalid relay invocations must fail before a window or network session starts.
use std::process::Command;

#[test]
fn relay_requires_an_explicit_endpoint_and_local_level_without_account_protocol() {
    for tail in [
        vec!["--online", "127.0.0.1"],
        vec!["--online", "127.0.0.1:14801"],
        vec!["--online", "127.0.0.1:14801", "--list-only"],
        vec!["--online", "127.0.0.1:14801", "--secure-login", "required"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_seihai-mae"))
            .args(["--version", "1.1.9 Gold Edition"])
            .args(&tail)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{tail:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("relay"),
            "{tail:?}"
        );
    }
}
