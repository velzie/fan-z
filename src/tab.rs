use egui::Ui;

use crate::FanzApp;

pub trait Tab<'a> {
    fn ui(&mut self, app: &mut FanzApp<'a>, ui: &mut Ui);
    fn name(&self) -> &str;
}
