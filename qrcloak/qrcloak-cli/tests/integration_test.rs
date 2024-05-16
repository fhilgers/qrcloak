use std::{
    env,
    process::{Command, Stdio},
};

use age::secrecy::ExposeSecret;

#[test]
fn test_filter() {
    let qrcloak_cli = env!("CARGO_BIN_EXE_qrcloak-cli");

    let id = age::x25519::Identity::generate();

    env::set_var("AGE_PRIVATE_KEY", id.to_string().expose_secret());
    env::set_var("AGE_KEY", id.to_public().to_string());

    let mut generate = Command::new(qrcloak_cli)
        .args([
            "payload",
            "generate",
            "--splits",
            "4",
            "--age-key",
            "--text",
            "hello world",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn qrcloak-cli generate");

    let mut merge = Command::new(qrcloak_cli)
        .args(["payload", "merge", "--file", "-"])
        .stdin(Stdio::from(generate.stdout.take().unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn qrcloak-cli merge");

    let extract = Command::new(qrcloak_cli)
        .args(["payload", "extract", "--age-key", "--file", "-"])
        .stdin(Stdio::from(merge.stdout.take().unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn qrcloak-cli extract");

    let res = generate
        .wait()
        .expect("failed to wait for qrcloak-cli generate");
    assert!(res.success());

    let res = merge.wait().expect("failed to wait for qrcloak-cli merge");
    assert!(res.success());

    let generate = extract
        .wait_with_output()
        .expect("failed to wait for qrcloak-cli extract");
    assert!(generate.status.success());

    assert_eq!(String::from_utf8_lossy(&generate.stdout), "hello world\n");
}
