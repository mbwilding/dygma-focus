use egui::{Slider, Ui};
use std::time::Duration;

pub(crate) fn slider_duration_millis(
    ui: &mut Ui,
    value: &mut Duration,
    range: std::ops::RangeInclusive<i32>,
) {
    let mut ms = value.as_millis() as i32;
    let slider = Slider::new(&mut ms, range).text("ms");
    if ui.add(slider).changed() {
        *value = Duration::from_millis(ms as u64);
    }
}

pub(crate) fn slider_duration_minutes(
    ui: &mut Ui,
    value: &mut Duration,
    range: std::ops::RangeInclusive<i32>,
    label: &str,
) {
    ui.label(label);
    let mut min = (value.as_secs() / 60) as i32;
    let slider = Slider::new(&mut min, range).text("min");
    if ui.add(slider).changed() {
        *value = Duration::from_secs((min * 60) as u64);
    }
}

pub(crate) fn slider_duration_minutes_option(
    ui: &mut Ui,
    value: &mut Option<Duration>,
    range: std::ops::RangeInclusive<i32>,
    label: &str,
) {
    if let Some(value) = value {
        slider_duration_minutes(ui, value, range, label)
    }
}
