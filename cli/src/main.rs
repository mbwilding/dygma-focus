use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use dygma_focus::prelude::*;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the version of the firmware
    Version,

    /// Settings commands
    Settings {
        #[command(subcommand)]
        subcommand: Settings,
    },

    /// Superkeys commands, durations are in milliseconds
    Superkeys {
        #[command(subcommand)]
        subcommand: Superkeys,
    },
}

#[derive(Subcommand)]
enum Settings {
    /// Get or set the default layer the keyboard will boot with
    DefaultLayer {
        #[arg(value_name = "VALUE")]
        value: Option<u8>,
    },

    /// Get a value that states true if all checks have been performed on the current settings, and its upload was done in the intended way
    Valid,

    /// Get or set the current settings version
    Version {
        #[arg(value_name = "VALUE")]
        value: Option<String>,
    },

    /// Get the CRC checksum of the layout
    Crc,

    /// Get the EEPROM's free bytes
    EepromFree,
}

#[derive(Subcommand)]
enum Superkeys {
    /// Wait for specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held
    WaitFor {
        #[arg(value_name = "VALUE")]
        value: Option<u64>,
    },

    /// The timeout of how long superkeys waits for the next tap
    Timeout {
        #[arg(value_name = "VALUE")]
        value: Option<u64>,
    },

    /// The repeat value is the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded
    Repeat {
        #[arg(value_name = "VALUE")]
        value: Option<u64>,
    },

    /// The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action
    HoldStart {
        #[arg(value_name = "VALUE")]
        value: Option<u64>,
    },

    /// The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the superkey
    Overlap {
        #[arg(value_name = "VALUE")]
        value: Option<u8>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut focus = Focus::new_first_available()?;

    if let Some(command) = cli.command {
        match command {
            Commands::Version => {
                let result = focus.version().await?;
                println!("{result}");
            }
            Commands::Settings { subcommand } => match subcommand {
                Settings::DefaultLayer { value } => {
                    if let Some(value) = value {
                        focus.settings_default_layer_set(value).await?;
                    } else {
                        let result = focus.settings_default_layer_get().await?;
                        println!("{result:?}");
                    }
                }
                Settings::Valid => {
                    let result = focus.settings_valid().await?;
                    println!("{result}");
                }
                Settings::Version { value } => {
                    if let Some(value) = value {
                        focus.settings_version_set(&value).await?;
                    } else {
                        let result = focus.settings_version_get().await?;
                        println!("{result:?}");
                    }
                }
                Settings::Crc => {
                    let result = focus.settings_crc().await?;
                    println!("{result}");
                }
                Settings::EepromFree => {
                    let result = focus.eeprom_free().await?;
                    println!("{result}");
                }
            },
            Commands::Superkeys { subcommand } => match subcommand {
                Superkeys::WaitFor { value } => {
                    if let Some(value) = value {
                        let value = Duration::from_millis(value);
                        focus.superkeys_wait_for_set(value).await?;
                    } else {
                        let result = focus.superkeys_wait_for_get().await?;
                        println!("{result:?}");
                    }
                }
                Superkeys::Timeout { value } => {
                    if let Some(value) = value {
                        let value = Duration::from_millis(value);
                        focus.superkeys_timeout_set(value).await?;
                    } else {
                        let result = focus.superkeys_timeout_get().await?;
                        println!("{result:?}");
                    }
                }
                Superkeys::Repeat { value } => {
                    if let Some(value) = value {
                        let value = Duration::from_millis(value);
                        focus.superkeys_repeat_set(value).await?;
                    } else {
                        let result = focus.superkeys_repeat_get().await?;
                        println!("{result:?}");
                    }
                }
                Superkeys::HoldStart { value } => {
                    if let Some(value) = value {
                        let value = Duration::from_millis(value);
                        focus.superkeys_hold_start_set(value).await?;
                    } else {
                        let result = focus.superkeys_hold_start_get().await?;
                        println!("{result:?}");
                    }
                }
                Superkeys::Overlap { value } => {
                    if let Some(value) = value {
                        focus.superkeys_overlap_set(value).await?;
                    } else {
                        let result = focus.superkeys_overlap_get().await?;
                        println!("{result:?}");
                    }
                }
            },
        }
    }

    Ok(())
}
