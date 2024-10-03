use crate::components;

use dygma_focus::prelude::*;
use egui::{ScrollArea, Slider};

pub struct App {
    pub focus: dygma_focus::Focus,
    pub settings: Settings,
    pub settings_original: Settings,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Panic if no device, this is just an example though
        let mut focus = Focus::new_first_available().unwrap();
        let settings = focus.settings_get().unwrap();

        Self {
            focus,
            settings: settings.clone(),
            settings_original: settings,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Save").clicked() {
                    let _ = self.focus.settings_set(&self.settings);
                }
                if ui.button("Reset").clicked() {
                    self.settings = self.settings_original.clone();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.collapsing("Typing and Keys", |ui| {
                    ui.collapsing("Typing", |ui| {
                        ui.collapsing("Dual-Function Keys", |ui| {
                            ui.label("Overlap Threshold");
                            components::slider_duration_millis(
                                ui,
                                &mut self.settings.qukeys_overlap_threshold,
                                0..=100,
                            );

                            ui.label("Hold Timeout");
                            components::slider_duration_millis(
                                ui,
                                &mut self.settings.qukeys_hold_timeout,
                                1..=255,
                            );
                        });

                        ui.collapsing("Super Keys", |ui| {
                            ui.label("Overlap Threshold");
                            ui.add(
                                Slider::new(&mut self.settings.superkeys_overlap, 0..=80).text("%"),
                            );

                            ui.label("Next Tap Timeout");
                            components::slider_duration_millis(
                                ui,
                                &mut self.settings.superkeys_timeout,
                                1..=500,
                            );

                            ui.label("Hold Timeout");
                            components::slider_duration_millis(
                                ui,
                                &mut self.settings.superkeys_hold_start,
                                120..=500,
                            );
                        });
                    });

                    ui.collapsing("Mouse", |ui| {
                        ui.label("Speed");
                        ui.add(Slider::new(&mut self.settings.mouse_speed, 0..=127).text("Speed"));

                        ui.label("Maximum Speed");
                        ui.add(
                            Slider::new(&mut self.settings.mouse_speed_limit, 0..=255)
                                .text("Speed"),
                        );

                        ui.label("Acceleration");
                        ui.add(
                            Slider::new(&mut self.settings.mouse_acceleration_speed, 0..=254)
                                .text("Accel"),
                        );

                        ui.label("Wheel Speed");
                        ui.add(
                            Slider::new(&mut self.settings.mouse_wheel_speed, 0..=255)
                                .text("Speed"),
                        );
                    });
                });

                ui.collapsing("LED", |ui| {
                    ui.collapsing("Brightness", |ui| {
                        ui.label("Keys");
                        ui.add(
                            Slider::new(&mut self.settings.led_brightness_keys_wired, 0..=255)
                                .text("Wired"),
                        );
                        if let Some(value) = &mut self.settings.led_brightness_keys_wireless {
                            let slider = Slider::new(value, 0..=255).text("Wireless");
                            if ui.add(slider).changed() {
                                self.settings.led_brightness_keys_wireless = Some(*value);
                            }
                        }

                        ui.label("Underglow");
                        if let Some(value) = &mut self.settings.led_brightness_underglow_wired {
                            let slider = Slider::new(value, 0..=255).text("Wired");
                            if ui.add(slider).changed() {
                                self.settings.led_brightness_underglow_wired = Some(*value);
                            }
                        }
                        if let Some(value) = &mut self.settings.led_brightness_underglow_wireless {
                            let slider = Slider::new(value, 0..=255).text("Wireless");
                            if ui.add(slider).changed() {
                                self.settings.led_brightness_underglow_wireless = Some(*value);
                            }
                        }
                    });

                    ui.collapsing("Sleep", |ui| {
                        components::slider_duration_minutes(
                            ui,
                            &mut self.settings.led_idle_time_limit_wired,
                            0..=60,
                            "Wired",
                        );
                        components::slider_duration_minutes_option(
                            ui,
                            &mut self.settings.led_idle_time_limit_wireless,
                            0..=60,
                            "Wireless",
                        );
                    });
                });

                ui.collapsing("Battery Management", |ui| {
                    if let Some(value) = &mut self.settings.led_idle_true_sleep {
                        ui.horizontal(|ui| {
                            ui.label("True Sleep");
                            ui.checkbox(value, "");
                        });
                    }
                    components::slider_duration_minutes_option(
                        ui,
                        &mut self.settings.led_idle_true_sleep_time,
                        0..=240,
                        "True Sleep delay after LEDs are off",
                    );
                });
            });
        });
    }
}
