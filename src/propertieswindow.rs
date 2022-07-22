use crate::zvm::{self, ZEvent, ZVMState, ZVM};
use crate::FanzApp;
use egui::{
    pos2, vec2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2,
    Widget,
};
pub struct PropertiesWindow {}
impl Default for PropertiesWindow {
    fn default() -> Self {
        PropertiesWindow {}
    }
}
impl PropertiesWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        let mut object = app.cart.objects[]
        ui.heading()
    }
}
