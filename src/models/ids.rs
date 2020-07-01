use thiserror::Error;

pub use super::mods::{ModId, VersionId};
pub use super::teams::{TeamId, UserId};

/// Generates a random 64 bit integer that is exactly `n` characters
/// long when encoded as base62.
///
/// Uses `rand`'s thread rng on every call.
///
/// # Panics
///
/// This method panics if `n` is 0 or greater than 11, since a `u64`
/// can only represent up to 11 character base62 strings
pub fn random_base62(n: usize) -> u64 {
    use rand::Rng;
    assert!(n > 0 && n <= 11);
    let mut rng = rand::thread_rng();
    // gen_range is [low, high): max value is `MULTIPLES[n] - 1`,
    // which is n characters long when encoded
    rng.gen_range(MULTIPLES[n - 1], MULTIPLES[n])
}

/// Generates a random 64 bit integer that is exactly `n` characters
/// long when encoded as base62, using the given rng.
///
/// # Panics
///
/// This method panics if `n` is 0 or greater than 11, since a `u64`
/// can only represent up to 11 character base62 strings
pub fn random_base62_rng<R: rand::RngCore>(rng: &mut R, n: usize) -> u64 {
    use rand::Rng;
    assert!(n > 0 && n <= 11);
    rng.gen_range(MULTIPLES[n - 1], MULTIPLES[n])
}

const MULTIPLES: [u64; 12] = [
    1,
    62,
    62 * 62,
    62 * 62 * 62,
    62 * 62 * 62 * 62,
    62 * 62 * 62 * 62 * 62,
    62 * 62 * 62 * 62 * 62 * 62,
    62 * 62 * 62 * 62 * 62 * 62 * 62,
    62 * 62 * 62 * 62 * 62 * 62 * 62 * 62,
    62 * 62 * 62 * 62 * 62 * 62 * 62 * 62 * 62,
    62 * 62 * 62 * 62 * 62 * 62 * 62 * 62 * 62 * 62,
    std::u64::MAX,
];

/// An ID encoded as base62 for use in the API.
///
/// All ids should be random and encode to 8-10 character base62 strings,
/// to avoid enumeration and other attacks.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Base62Id(pub u64);

/// An error decoding a number from base62.
#[derive(Error, Debug)]
pub enum DecodingError {
    /// Encountered a non base62 character in base62 string
    #[error("Invalid character `{0:?}` in base62 encoding")]
    InvalidBase62(char),
    /// Encountered integer overflow when decoding a base62 id.
    #[error("Base62 decoding overflowed")]
    Overflow,
}

macro_rules! from_base62id {
    ($($struct:ty, $con:expr;)+) => {
        $(
            impl From<Base62Id> for $struct {
                fn from(id: Base62Id) -> $struct {
                    $con(id.0)
                }
            }
            impl From<$struct> for Base62Id {
                fn from(id: $struct) -> Base62Id {
                    Base62Id(id.0)
                }
            }
        )+
    };
}

from_base62id! {
    ModId, ModId;
    UserId, UserId;
    VersionId, VersionId;
    TeamId, TeamId;
}

pub mod base62_impl {
    use serde::de::{self, Deserializer, Visitor};
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};

    use super::{Base62Id, DecodingError};

    impl<'de> Deserialize<'de> for Base62Id {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Base62Visitor;

            impl<'de> Visitor<'de> for Base62Visitor {
                type Value = Base62Id;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a base62 string id")
                }

                fn visit_str<E>(self, string: &str) -> Result<Base62Id, E>
                where
                    E: de::Error,
                {
                    parse_base62(string).map(Base62Id).map_err(E::custom)
                }
            }

            deserializer.deserialize_str(Base62Visitor)
        }
    }

    impl Serialize for Base62Id {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&to_base62(self.0))
        }
    }

    const BASE62_CHARS: [u8; 62] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
        b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i',
        b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
        b'y', b'z',
    ];

    pub fn to_base62(mut num: u64) -> String {
        let length = (num as f64).log(62.0).ceil() as usize;
        let mut output = String::with_capacity(length);

        while num > 0 {
            // Could be done more efficiently, but requires byte
            // manipulation of strings & Vec<u8> -> String conversion
            output.insert(0, BASE62_CHARS[(num % 62) as usize] as char);
            num /= 62;
        }
        output
    }

    fn parse_base62(string: &str) -> Result<u64, DecodingError> {
        let mut num: u64 = 0;
        for c in string.chars().rev() {
            let next_digit;
            if c.is_ascii_digit() {
                next_digit = (c as u8 - b'0') as u64;
            } else if c.is_ascii_uppercase() {
                next_digit = 10 + (c as u8 - b'A') as u64;
            } else if c.is_ascii_lowercase() {
                next_digit = 36 + (c as u8 - b'a') as u64;
            } else {
                return Err(DecodingError::InvalidBase62(c));
            }

            // We don't want this panicing or wrapping on integer overflow
            if let Some(n) = num.checked_mul(62).and_then(|n| n.checked_add(next_digit)) {
                num = n;
            } else {
                return Err(DecodingError::Overflow);
            }
        }
        Ok(num)
    }
}
