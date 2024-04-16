use core::panic;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{env, str::FromStr};

use age::x25519::Identity;
use indoc::formatdoc;
use qrcloak_core::payload::{AgeKeyDecryption, Decryption, PayloadExtractor, PayloadMerger};
use tempdir::TempDir;

fn block_encryption<'a>(
    public_keys: impl AsRef<[&'a str]>,
    path: &str,
    alt_name: &str,
    data: &str,
) -> String {
    formatdoc!(
        r#"
        ```{{#qrcloak age-keys="{age_public_key}" path="{path}" alt-name="{name}"}}
        {data}
        ```
        "#,
        age_public_key = public_keys.as_ref().join(","),
        path = path,
        name = alt_name,
        data = data,
    )
}

pub fn cargo_dir() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
        })
        .expect("could not find executable path")
}

#[test]
fn test_filter() {
    let filter_bin = env!("CARGO_BIN_EXE_qrcloak-pandoc");

    let age_public_key = "age1ny7dkv07ftpvu3tfxfjkaf5smuhdvs9uhjlptejte5ruadlw44psqhgzsw";
    let age_private_key =
        "AGE-SECRET-KEY-17ZWSEMWJ4TVXG2ZD7TJ7275GERE0X2AWQ7FUD9JJ3EXY303QRDWQRNQJ8A";

    let tmp_dir = TempDir::new("test_filter").expect("could not create temp dir");
    let qrcode_path = tmp_dir.path().join("qrcode.png");

    let data = "Input to encrypt";

    let simple_block = block_encryption(
        &[age_public_key],
        qrcode_path.to_string_lossy().as_ref(),
        "First Block",
        data,
    );

    let input_path = tmp_dir.path().join("file.md");
    let output_path = tmp_dir.path().join("output.md");

    let mut markdown_file = File::create(&input_path).expect("could not create file");
    writeln!(markdown_file, "{}", simple_block).expect("could not write to file");

    let output = Command::new("pandoc")
        .arg(format!("{}", input_path.display()))
        .arg(format!("--output={}", output_path.display()))
        .arg(format!("--filter={}", filter_bin))
        .output()
        .expect("could not run pandoc");

    if !output.status.success() {
        panic!("pandoc failed with status {:?}", output.status);
    }

    let qrcode_image = image::open(&qrcode_path)
        .expect("could not open qrcode image")
        .to_luma8();

    let (width, height) = (
        qrcode_image.width() as usize,
        qrcode_image.height() as usize,
    );

    let payload = qrcloak_core::extract::Extractor::extract(width, height, &*qrcode_image);

    assert_eq!(payload.len(), 1);

    let merged = PayloadMerger::default().merge(payload).0;

    let payload = PayloadExtractor::default()
        .with_decryption(Decryption::AgeKey(AgeKeyDecryption::new(vec![
            Identity::from_str(age_private_key).expect("should be valid key"),
        ])))
        .extract(merged[0].clone())
        .expect("should extract");

    assert_eq!(payload, data);

    let mut output_file = File::open(output_path).expect("could not open output file");
    let mut output_data = String::new();
    output_file
        .read_to_string(&mut output_data)
        .expect("could not read output file");

    assert_eq!(
        output_data,
        format!(
            "![]({} \"First Block\"){{#qrcloak}}\n",
            qrcode_path.display()
        )
    );
}
