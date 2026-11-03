//! Icon editor configurations shared by instances, servers, and shared instances.

use serde::{Deserialize, Serialize};

/// The maximum length of an icon symbol ID.
pub const MAX_ICON_SYMBOL_LENGTH: usize = 64;

/// The background drawn behind an icon's symbol.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum IconBackground {
    /// A solid `#RRGGBB` color.
    Color { value: String },
    /// A gradient from `top_color` at the top edge to `bottom_color` at the bottom edge.
    #[serde(rename = "linear-top-down-gradient")]
    LinearTopDownGradient {
        top_color: String,
        bottom_color: String,
    },
}

/// An icon created with the icon editor: a symbol drawn over a background.
///
/// Symbol IDs are defined by the frontend's symbol catalog, so only their format is validated.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IconConfig {
    pub background: IconBackground,
    pub symbol: String,
}

/// An [`IconBackground`] with its colors parsed into RGB components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatedIconBackground {
    Color([u8; 3]),
    LinearTopDownGradient {
        top_color: [u8; 3],
        bottom_color: [u8; 3],
    },
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum IconConfigError {
    #[error("icon background must be a hexadecimal color")]
    InvalidColor,
    #[error("icon symbol ID is invalid")]
    InvalidSymbol,
}

impl IconConfig {
    /// Validates the background colors and symbol ID format, returning the parsed background.
    pub fn validate(&self) -> Result<ValidatedIconBackground, IconConfigError> {
        let background = self.background.validate()?;
        if !is_valid_symbol_id(&self.symbol) {
            return Err(IconConfigError::InvalidSymbol);
        }
        Ok(background)
    }
}

impl IconBackground {
    pub fn validate(&self) -> Result<ValidatedIconBackground, IconConfigError> {
        Ok(match self {
            Self::Color { value } => {
                ValidatedIconBackground::Color(parse_hex_color(value)?)
            }
            Self::LinearTopDownGradient {
                top_color,
                bottom_color,
            } => ValidatedIconBackground::LinearTopDownGradient {
                top_color: parse_hex_color(top_color)?,
                bottom_color: parse_hex_color(bottom_color)?,
            },
        })
    }
}

fn parse_hex_color(value: &str) -> Result<[u8; 3], IconConfigError> {
    let hex = value
        .strip_prefix('#')
        .filter(|hex| {
            hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
        .ok_or(IconConfigError::InvalidColor)?;
    let color = u32::from_str_radix(hex, 16)
        .map_err(|_| IconConfigError::InvalidColor)?;
    Ok([(color >> 16) as u8, (color >> 8) as u8, color as u8])
}

fn is_valid_symbol_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ICON_SYMBOL_LENGTH
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(background: IconBackground, symbol: &str) -> IconConfig {
        IconConfig {
            background,
            symbol: symbol.to_string(),
        }
    }

    fn color(value: &str) -> IconBackground {
        IconBackground::Color {
            value: value.to_string(),
        }
    }

    #[test]
    fn validates_color_and_symbol_id() {
        assert_eq!(
            config(color("#c78aff"), "dusk_block").validate(),
            Ok(ValidatedIconBackground::Color([199, 138, 255]))
        );
        assert_eq!(
            config(color("purple"), "dusk_block").validate(),
            Err(IconConfigError::InvalidColor)
        );
        assert_eq!(
            config(color("#+12345"), "dusk_block").validate(),
            Err(IconConfigError::InvalidColor)
        );
        assert_eq!(
            config(color("#c78aff"), "dusk-block").validate(),
            Err(IconConfigError::InvalidSymbol)
        );
    }

    #[test]
    fn serializes_in_the_shared_format() {
        let config = config(
            IconBackground::LinearTopDownGradient {
                top_color: "#D62E63".to_string(),
                bottom_color: "#F95C62".to_string(),
            },
            "grass_block",
        );
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "background": {
                    "type": "linear-top-down-gradient",
                    "top_color": "#D62E63",
                    "bottom_color": "#F95C62",
                },
                "symbol": "grass_block",
            })
        );
        assert_eq!(serde_json::from_value::<IconConfig>(json).unwrap(), config);
    }
}
