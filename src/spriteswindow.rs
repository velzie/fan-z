use crate::zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM};
use crate::FanzApp;
use egui::{
    pos2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2, Widget,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct SpritesWindow {
    pub enabled: bool,
}
impl Default for SpritesWindow {
    fn default() -> Self {
        SpritesWindow { enabled: false }
    }
}
impl SpritesWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.button("fix 2");
        });
    }
}
