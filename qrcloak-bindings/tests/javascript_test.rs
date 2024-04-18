use std::process::Command;

#[test]
pub fn test_js() {
    let output = Command::new("just")
        .arg("test_js")
        .output()
        .expect("js test failed");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}
