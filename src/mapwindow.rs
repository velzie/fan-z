use crate::zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM};
use crate::FanzApp;
use egui::{
    pos2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2, Widget,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct MapWindow {
    pub enabled: bool,
}
impl Default for MapWindow {
    fn default() -> Self {
        MapWindow { enabled: false }
    }
}
impl MapWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.button("fix");
        });
    }
}
