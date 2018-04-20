#[derive(PartialEq, Debug)]
pub struct Info {
    pub number: String,
}

/// Decoder for some absurd format
pub struct Decoder;

impl Decoder {
    pub fn decode(mut data: &[u8]) -> Option<Info> {
        if data.len() < 1 {
            return None;
        }

        let len = data[0] as usize;
        if len > 10 {
            // Update: `cargo-fuzz` does be able to detect this bug.
            // See https://github.com/rust-fuzz/cargo-fuzz/issues/145
            panic!("I bet cargo-fuzz can't detect this.");
        }
        data = &data[1..];

        if data.len() < len {
            return None;
        }

        let mut acc = 0;
        for i in 0 .. len {
            acc += data[i];
        }

        Some(Info {
            number: format!("{}", acc)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let data = &[3u8, 1u8, 2u8, 3u8];
        assert_eq!(Decoder::decode(data).unwrap(),
            Info {
                number: "6".to_owned()
            });
    }

    #[test]
    #[should_panic(expected = "I bet cargo-fuzz can't detect this.")]
    fn length_too_long() {
        let data = &[11u8, 0u8, 0u8, 0u8];
        Decoder::decode(data);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn overflow() {
        let data = &[3u8, 100u8, 100u8, 100u8];
        Decoder::decode(data);
    }
}
