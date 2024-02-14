include!(concat!(env!("OUT_DIR"), "/wire_format.v1.rs"));

#[cfg(test)]
mod test {

    use prost::Message;

    use super::*;

    #[test]
    fn normal_payload() {
        let data = b"Hello world";
        let payload = NormalPayload { data: data.into() };

        let encoded = payload.encode_to_vec();

        insta::assert_yaml_snapshot!(encoded)
    }

    #[test]
    fn encrypted_payload() {
        let data = b"Hello world";
        let spec = encrypted_payload::Spec::Age(encrypted_payload::AgeSpec {});
        let payload = EncryptedPayload {
            data: data.into(),
            spec: Some(spec),
        };

        let encoded = payload.encode_to_vec();

        insta::assert_yaml_snapshot!(encoded)
    }

    #[test]
    fn normal_multi_payload() {
        let data = b"Hello world";
        let id = 5u32;

        let mid = data.len() / 2;

        let (first, second) = (&data[..mid], &data[mid..]);

        let first_payload = NormalPayload { data: first.into() };
        let second_payload = NormalPayload {
            data: second.into(),
        };

        let first_multi = MultiPayload {
            id,
            number: 0,
            total: 2,
            inner: Some(multi_payload::Inner::NormalPayload(first_payload)),
        };

        let second_multi = MultiPayload {
            id,
            number: 1,
            total: 2,
            inner: Some(multi_payload::Inner::NormalPayload(second_payload)),
        };

        let first_encoded = first_multi.encode_to_vec();
        let second_encoded = second_multi.encode_to_vec();

        insta::assert_yaml_snapshot!((first_encoded, second_encoded));
    }
}
