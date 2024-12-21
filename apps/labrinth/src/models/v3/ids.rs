pub use super::collections::CollectionId;
pub use super::images::ImageId;
pub use super::notifications::NotificationId;
pub use super::oauth_clients::OAuthClientAuthorizationId;
pub use super::oauth_clients::{OAuthClientId, OAuthRedirectUriId};
pub use super::organizations::OrganizationId;
pub use super::pats::PatId;
pub use super::payouts::PayoutId;
pub use super::projects::{ProjectId, VersionId};
pub use super::reports::ReportId;
pub use super::sessions::SessionId;
pub use super::teams::TeamId;
pub use super::threads::ThreadId;
pub use super::threads::ThreadMessageId;
pub use super::users::UserId;
pub use crate::models::billing::{
    ChargeId, ProductId, ProductPriceId, UserSubscriptionId,
};
use thiserror::Error;

/// Generates a random 64 bit integer that is exactly `n` characters
/// long when encoded as base62.
///
/// Uses `rand`'s thread rng on every call.
///
/// # Panics
///
/// This method panics if `n` is 0 or greater than 11, since a `u64`
/// can only represent up to 11 character base62 strings
#[inline]
pub fn random_base62(n: usize) -> u64 {
    random_base62_rng(&mut rand::thread_rng(), n)
}

/// Generates a random 64 bit integer that is exactly `n` characters
/// long when encoded as base62, using the given rng.
///
/// # Panics
///
/// This method panics if `n` is 0 or greater than 11, since a `u64`
/// can only represent up to 11 character base62 strings
pub fn random_base62_rng<R: rand::RngCore>(rng: &mut R, n: usize) -> u64 {
    random_base62_rng_range(rng, n, n)
}

pub fn random_base62_rng_range<R: rand::RngCore>(
    rng: &mut R,
    n_min: usize,
    n_max: usize,
) -> u64 {
    use rand::Rng;
    assert!(n_min > 0 && n_max <= 11 && n_min <= n_max);
    // gen_range is [low, high): max value is `MULTIPLES[n] - 1`,
    // which is n characters long when encoded
    rng.gen_range(MULTIPLES[n_min - 1]..MULTIPLES[n_max])
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
    u64::MAX,
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
    /// Encountered a non-base62 character in a base62 string
    #[error("Invalid character {0:?} in base62 encoding")]
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

macro_rules! impl_base62_display {
    ($struct:ty) => {
        impl std::fmt::Display for $struct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&base62_impl::to_base62(self.0))
            }
        }
    };
}
impl_base62_display!(Base62Id);

macro_rules! base62_id_impl {
    ($struct:ty, $cons:expr) => {
        from_base62id!($struct, $cons;);
        impl_base62_display!($struct);
    }
}
base62_id_impl!(ProjectId, ProjectId);
base62_id_impl!(UserId, UserId);
base62_id_impl!(VersionId, VersionId);
base62_id_impl!(CollectionId, CollectionId);
base62_id_impl!(TeamId, TeamId);
base62_id_impl!(OrganizationId, OrganizationId);
base62_id_impl!(ReportId, ReportId);
base62_id_impl!(NotificationId, NotificationId);
base62_id_impl!(ThreadId, ThreadId);
base62_id_impl!(ThreadMessageId, ThreadMessageId);
base62_id_impl!(SessionId, SessionId);
base62_id_impl!(PatId, PatId);
base62_id_impl!(ImageId, ImageId);
base62_id_impl!(OAuthClientId, OAuthClientId);
base62_id_impl!(OAuthRedirectUriId, OAuthRedirectUriId);
base62_id_impl!(OAuthClientAuthorizationId, OAuthClientAuthorizationId);
base62_id_impl!(PayoutId, PayoutId);
base62_id_impl!(ProductId, ProductId);
base62_id_impl!(ProductPriceId, ProductPriceId);
base62_id_impl!(UserSubscriptionId, UserSubscriptionId);
base62_id_impl!(ChargeId, ChargeId);

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

            impl Visitor<'_> for Base62Visitor {
                type Value = Base62Id;

                fn expecting(
                    &self,
                    formatter: &mut std::fmt::Formatter,
                ) -> std::fmt::Result {
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

    const BASE62_CHARS: [u8; 62] =
        *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

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

    pub fn parse_base62(string: &str) -> Result<u64, DecodingError> {
        let mut num: u64 = 0;
        for c in string.chars() {
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

            // We don't want this panicking or wrapping on integer overflow
            if let Some(n) =
                num.checked_mul(62).and_then(|n| n.checked_add(next_digit))
            {
                num = n;
            } else {
                return Err(DecodingError::Overflow);
            }
        }
        Ok(num)
    }
}
