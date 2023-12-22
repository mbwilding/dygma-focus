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

    println!(
        "Battery Level Left: {}",
        &focus.wireless_battery_level_left_get()?
    );

    println!(
        "Battery Level Right: {}",
        &focus.wireless_battery_level_right_get()?
    );

    println!(
        "Battery Left Status: {}",
        &focus.wireless_battery_status_left_get()?
    );

    println!(
        "Battery Right Status: {}",
        &focus.wireless_battery_status_right_get()?
    );

    println!(
        "Battery Saving Mode: {}",
        &focus.wireless_battery_saving_mode_get()?
    );

    Ok(())
}
