use anyhow::Result;
use dygma_focus::enums::*;
use tracing_subscriber::filter::LevelFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_ansi(true)
        .compact()
        .init();

    let mut focus = dygma_focus::Focus::default();
    focus.open_first()?;

    //focus.layer_move_to(3)?;
    // let response = focus.layer_is_active()?;
    // let response = focus.version()?;

    // println!("{:#?}", &focus.help()?);
    // println!("{:?}", &focus.layer_is_active()?);
    // println!("{:?}", &focus.layer_state()?);

    //println!("{:?}", &focus.version()?);

    // let response = focus.keymap_custom_get()?;
    // println!("{:?}", &response);
    // &focus.keymap_custom_set(&response)?;

    // let response = focus.keymap_default_get()?;
    // println!("{:?}", &response);
    // &focus.keymap_default_set(&response)?;

    // let response = focus.keymap_only_custom_get()?;
    // println!("{:?}", &response);
    // &focus.keymap_only_custom_set(true)?;

    // let response = focus.setting_default_layer_get()?;
    // println!("{:?}", &response);

    // let response = focus.settings_valid_get()?;
    // println!("{:?}", &response);

    // focus.settings_version_set(1)?;
    // println!("{:?}", &focus.settings_version_get()?);

    // println!("{:?}", &focus.settings_crc()?);

    // println!("{:?}", &focus.eeprom_contents_get()?);

    // println!("{:?}", &focus.eeprom_free_get()?);

    // let response = focus.led_at_get(0)?;
    // println!("{:?}", &response);
    // focus.led_at_set(0, Color {
    //     r: 255,
    //     g: 58,
    //     b: 0,
    // })?;

    // let response = focus.led_multiple_get(&vec![1, 2, 3, 4, 5])?;
    // println!("{:?}", &response);

    // println!("{:#?}", &focus.layer_state()?);

    focus.led_mode_set(LedMode::Standard)?;
    println!("{:?}", &focus.led_mode_get()?);

    Ok(())
}
