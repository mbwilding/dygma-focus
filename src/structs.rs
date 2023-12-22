use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        if parts.len() == 3 {
            Ok(Color {
                r: parts[0],
                g: parts[1],
                b: parts[2],
            })
        } else {
            Err(Error::msg("Invalid color format"))
        }
    }
}
