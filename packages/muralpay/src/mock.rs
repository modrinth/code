//! See [`MuralPayMock`].

use std::fmt::{self, Debug};

use crate::{Account, AccountId, MuralError};

macro_rules! impl_mock {
    (
        $(fn $fn:ident ( $( $ty:ty ),* ) -> $ret:ty);* $(;)?
    ) => {
        /// Mock data returned by [`crate::MuralPay`].
        pub struct MuralPayMock {
            $(
            pub(crate) $fn: Box<dyn Fn($($ty),*) -> $ret>,
            )*
        }

        impl Default for MuralPayMock {
            fn default() -> Self {
                Self {
                    $(
                    $fn: Box::new(|$(_: $ty),*| panic!("missing mock for `{}`", stringify!($fn))),
                    )*
                }
            }
        }
    };
}

impl_mock! {
    fn get_all_accounts() -> Result<Vec<Account>, MuralError>;
    fn get_account(AccountId) -> Result<Account, MuralError>;
    fn create_account(&str, Option<&str>) -> Result<Account, MuralError>;
}

impl Debug for MuralPayMock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MuralPayMock").finish_non_exhaustive()
    }
}
