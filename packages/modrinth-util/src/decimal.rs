use std::{
    cmp,
    ops::{Add, Sub},
};

use derive_more::{Deref, Display, Error};
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deref,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(try_from = "Decimal")]
pub struct DecimalDp<const DP: u32>(Decimal);

pub type Decimal2dp = DecimalDp<2>;

#[derive(Debug, Display, Clone, Error)]
#[display("decimal is not rounded to {dp} decimal places")]
pub struct NotRounded {
    pub dp: u32,
}

impl<const DP: u32> DecimalDp<DP> {
    pub const ZERO: Self = Self(Decimal::ZERO);

    pub fn rounded(v: Decimal, strategy: RoundingStrategy) -> Self {
        Self(v.round_dp_with_strategy(DP, strategy))
    }

    pub fn new(v: Decimal) -> Result<Self, NotRounded> {
        if v.round_dp(DP) == v {
            Ok(Self(v))
        } else {
            Err(NotRounded { dp: DP })
        }
    }

    pub fn get(self) -> Decimal {
        self.0
    }

    pub fn mul_round(
        self,
        other: impl Into<Decimal>,
        strategy: RoundingStrategy,
    ) -> Self {
        Self::rounded(self.0 * other.into(), strategy)
    }
}

// conversion

impl<const DP: u32> TryFrom<Decimal> for DecimalDp<DP> {
    type Error = NotRounded;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const DP: u32> From<DecimalDp<DP>> for Decimal {
    fn from(value: DecimalDp<DP>) -> Self {
        value.0
    }
}

// ord

impl<const DP: u32> PartialOrd<Decimal> for DecimalDp<DP> {
    fn partial_cmp(&self, other: &Decimal) -> Option<cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<const DP: u32> PartialOrd<DecimalDp<DP>> for Decimal {
    fn partial_cmp(&self, other: &DecimalDp<DP>) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

// eq

impl<const DP: u32> PartialEq<Decimal> for DecimalDp<DP> {
    fn eq(&self, other: &Decimal) -> bool {
        self.0.eq(other)
    }
}

impl<const DP: u32> PartialEq<DecimalDp<DP>> for Decimal {
    fn eq(&self, other: &DecimalDp<DP>) -> bool {
        self.eq(&other.0)
    }
}

// add

impl<const DP: u32> Add for DecimalDp<DP> {
    type Output = Self;

    fn add(self, rhs: DecimalDp<DP>) -> Self::Output {
        let v = self.0 + rhs.0;
        debug_assert!(Self::new(v).is_ok());
        Self(v)
    }
}

impl<const DP: u32> Add<Decimal> for DecimalDp<DP> {
    type Output = Decimal;

    fn add(self, rhs: Decimal) -> Self::Output {
        self.0 + rhs
    }
}

impl<const DP: u32> Add<DecimalDp<DP>> for Decimal {
    type Output = Decimal;

    fn add(self, rhs: DecimalDp<DP>) -> Self::Output {
        self + rhs.0
    }
}

// sub

impl<const DP: u32> Sub for DecimalDp<DP> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let v = self.0 - rhs.0;
        debug_assert!(Self::new(v).is_ok());
        Self(v)
    }
}

impl<const DP: u32> Sub<Decimal> for DecimalDp<DP> {
    type Output = Decimal;

    fn sub(self, rhs: Decimal) -> Self::Output {
        self.0 - rhs
    }
}

impl<const DP: u32> Sub<DecimalDp<DP>> for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: DecimalDp<DP>) -> Self::Output {
        self - rhs.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal::dec;

    #[test]
    fn new() {
        Decimal2dp::new(dec!(1)).unwrap();
        Decimal2dp::new(dec!(1.0)).unwrap();
        Decimal2dp::new(dec!(1.1)).unwrap();
        Decimal2dp::new(dec!(1.01)).unwrap();
        Decimal2dp::new(dec!(1.00)).unwrap();
        Decimal2dp::new(dec!(1.000)).unwrap();
        Decimal2dp::new(dec!(1.001)).unwrap_err();
    }

    #[test]
    fn rounded() {
        assert_eq!(
            dec!(1),
            Decimal2dp::rounded(dec!(1), RoundingStrategy::ToZero)
        );
        assert_eq!(
            dec!(1),
            Decimal2dp::rounded(dec!(1.001), RoundingStrategy::ToZero)
        );
        assert_eq!(
            dec!(1),
            Decimal2dp::rounded(dec!(1.005), RoundingStrategy::ToZero)
        );
        assert_eq!(
            dec!(1),
            Decimal2dp::rounded(dec!(1.009), RoundingStrategy::ToZero)
        );
        assert_eq!(
            dec!(1.01),
            Decimal2dp::rounded(dec!(1.010), RoundingStrategy::ToZero)
        );
    }

    #[test]
    fn deserialize() {
        serde_json::from_str::<Decimal2dp>("1").unwrap();
        serde_json::from_str::<Decimal2dp>("1.0").unwrap();
        serde_json::from_str::<Decimal2dp>("1.00").unwrap();
        serde_json::from_str::<Decimal2dp>("1.000").unwrap();
        serde_json::from_str::<Decimal2dp>("1.001").unwrap_err();
    }

    #[test]
    fn ops() {
        assert_eq!(
            Decimal2dp::new(dec!(1.23)).unwrap()
                + Decimal2dp::new(dec!(0.27)).unwrap(),
            dec!(1.50)
        );
        assert_eq!(
            Decimal2dp::new(dec!(1.23)).unwrap()
                - Decimal2dp::new(dec!(0.23)).unwrap(),
            dec!(1.00)
        );
    }
}
