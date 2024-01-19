use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// RGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RGB {
    /// Red component of the color.
    pub r: u8,
    /// Green component of the color.
    pub g: u8,
    /// Blue component of the color.
    pub b: u8,
}

impl FromStr for RGB {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        if parts.len() == 3 {
            Ok(Self {
                r: parts[0],
                g: parts[1],
                b: parts[2],
            })
        } else {
            bail!("Invalid color format");
        }
    }
}

/// RGBA color, the alpha is currently ignored by the firmware.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RGBA {
    /// Red component of the color.
    pub r: u8,
    /// Green component of the color.
    pub g: u8,
    /// Blue component of the color.
    pub b: u8,
    /// Alpha component of the color, currently ignored by the firmware.
    pub a: u8,
}

impl FromStr for RGBA {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        if parts.len() == 4 {
            Ok(Self {
                r: parts[0],
                g: parts[1],
                b: parts[2],
                a: parts[3],
            })
        } else {
            bail!("Invalid color format");
        }
    }
}
