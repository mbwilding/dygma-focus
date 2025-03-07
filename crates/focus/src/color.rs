use crate::errors::FocusError;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The LED RGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RGB {
    /// Red component of the color.
    pub r: u8,
    /// Green component of the color.
    pub g: u8,
    /// Blue component of the color.
    pub b: u8,
}

impl FromStr for RGB {
    type Err = FocusError;

    fn from_str(s: &str) -> Result<Self, FocusError> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        let parts_len = parts.len();
        if parts_len == 3 {
            Ok(Self {
                r: parts[0],
                g: parts[1],
                b: parts[2],
            })
        } else {
            Err(FocusError::PartCountError { expected: 3 })
        }
    }
}

/// The LED RGBW color.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RGBW {
    /// Red component of the color.
    pub r: u8,
    /// Green component of the color.
    pub g: u8,
    /// Blue component of the color.
    pub b: u8,
    /// White component of the color.
    pub w: u8,
}

impl FromStr for RGBW {
    type Err = FocusError;

    fn from_str(s: &str) -> Result<Self, FocusError> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        let parts_len = parts.len();
        if parts_len == 4 {
            Ok(Self {
                r: parts[0],
                g: parts[1],
                b: parts[2],
                w: parts[3],
            })
        } else {
            Err(FocusError::PartCountError { expected: 4 })
        }
    }
}
