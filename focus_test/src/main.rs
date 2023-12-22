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

    println!("{:#?}", &focus.help_get()?);

    println!("{:#?}", &focus.led_mode_get()?);

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

    println!("Wireless RF Power: {:?}", &focus.wireless_rf_power_get()?);

    println!(
        "Wireless RF Channel Hop: {}",
        &focus.wireless_rf_channel_hop_get()?
    );

    println!("Mouse Speed: {}", focus.mouse_speed_get()?);

    println!("Mouse Delay: {}", focus.mouse_delay_get()?);

    println!(
        "Mouse Acceleration Speed: {}",
        focus.mouse_acceleration_speed_get()?
    );

    println!(
        "Mouse Acceleration Delay: {}",
        focus.mouse_acceleration_delay_get()?
    );

    println!("Mouse Wheel Speed: {}", focus.mouse_wheel_speed_get()?);

    println!(
        "Mouse Wheel Speed Delay: {}",
        focus.mouse_wheel_delay_get()?
    );

    println!("Mouse Speed Limit: {}", focus.mouse_speed_limit_get()?);

    println!("Macros Memory: {}", focus.macros_memory_get()?);

    println!("Idle LED True Sleep: {}", focus.led_idle_true_sleep_get()?);

    println!(
        "Idle LED True Sleep Time: {}",
        focus.led_idle_true_sleep_time_get()?
    );

    Ok(())
}
