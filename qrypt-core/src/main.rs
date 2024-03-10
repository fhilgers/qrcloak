use qrypt_core::encrypt_to_b64;

pub mod wire_format;

fn main() {
    let result = encrypt_to_b64("hello world", "password").unwrap();

    println!("{result}");
}
