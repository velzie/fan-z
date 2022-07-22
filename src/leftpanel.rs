use std::rc::Rc;
use std::vec;

use crate::app::{draw_sprite, EditorObject};
use crate::tab::Tab;
use crate::zvm::{self, ZEvent, ZVMState, ZVM};
use crate::FanzApp;
use egui::{
    pos2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2, Widget,
};
pub struct SpritesSelector;
impl<'a> Tab<'a> for SpritesSelector {
    fn name(&self) -> &str {
        "Sprites"
    }
    fn ui(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (i, sprite) in app.cart.sprites.iter().enumerate() {
                let (response, painter) =
                    ui.allocate_painter(Vec2::new(32.0, 32.0), Sense::click());
                let rect = painter.clip_rect();

                ui.painter().rect(
                    rect,
                    0f32,
                    Color32::BLACK,
                    if i == app.selectedsprite {
                        Stroke::new(2.0, Color32::WHITE)
                    } else {
                        Stroke::none()
                    },
                );
                draw_sprite(&painter, rect, sprite);
                if response.clicked() {
                    app.selectedsprite = i;
                }
            }
        });
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.label(format!("Selected Sprite: {}", app.selectedsprite))
        });
    }
}
pub struct ObjectSelector;
impl<'a> Tab<'a> for ObjectSelector {
    fn name(&self) -> &str {
        "Objects"
    }
    fn ui(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        if ui.button("add object").clicked() {
            app.cart.objects.push(EditorObject {
                pos: pos2(0.0, 0.0),
                name: format!("Object {}", app.cart.objects.len()),
                script: "put \"Hello World\"".into(),
            });
        }
        ui.vertical(|ui| {
            for (i, o) in app.cart.objects.iter().enumerate() {
                if ui.button(&o.name).clicked() {
                    app.selectedobject = i;
                }
            }
        });
    }
}
