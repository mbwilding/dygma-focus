use anyhow::Result;
use dygma_focus::prelude::*;
use tracing_subscriber::filter::LevelFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_ansi(true)
        .compact()
        .init();

    let mut focus = dygma_focus::Focus::default();
    focus.device_open_first()?;

    // println!("{:#?}", &focus.help_get()?);
    // println!("{:#?}", &focus.led_mode_get()?);

    focus.led_mode_set(LedMode::PerLayer)?;

    Ok(())
}
