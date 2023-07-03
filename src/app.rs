#![allow(unused_must_use)]
use crate::{
    codewindow::CodeWindow,
    gamewindow::GameWindow,
    leftpanel::{ObjectSelector, SpritesSelector},
    mapwindow::MapWindow,
    propertieswindow::PropertiesWindow,
    spriteswindow::{sized_toolbtn_ui, Sprite, SpritesWindow},
    tab::Tab,
    zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM},
};
use array2d::Array2D;
use egui::{
    pos2, vec2, Align2, Color32, Id, Key, LayerId, Layout, Painter, Pos2, Rect, RichText, Sense,
    Stroke, Vec2, Widget,
};
use std::{cell::RefCell, fmt::Debug, mem, rc::Rc, sync::Mutex};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
// #[serde(skip)]
pub struct FanzApp<'a> {
    pub cart: Cart,

    #[serde(skip)]
    pub codewindow: Rc<RefCell<CodeWindow>>,
    #[serde(skip)]
    pub mapwindow: Rc<RefCell<MapWindow>>,
    #[serde(skip)]
    pub spriteswindow: Rc<RefCell<SpritesWindow>>,

    #[serde(skip)]
    pub propertieswindow: Rc<RefCell<PropertiesWindow>>,
    #[serde(skip)]
    pub selectedsprite: usize,
    #[serde(skip)]
    pub selectedobject: usize,

    #[serde(skip)]
    pub gamewindow: Rc<RefCell<GameWindow<'a>>>,
    #[serde(skip)]
    pub leftpanelselected: usize,
    #[serde(skip)]
    pub leftpanel: Rc<RefCell<Vec<Box<dyn Tab<'a>>>>>,
    #[serde(skip)]
    pub output: Vec<RichText>,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Cart {
    pub sprites: Vec<Sprite>,
    pub map: Array2D<Option<usize>>,
    pub objects: Vec<EditorObject>,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditorObject {
    pub pos: Pos2,
    pub name: String,
    pub script: String,
}

impl<'a> Default for FanzApp<'a> {
    fn default() -> Self {
        dbg!(vec![
            Box::new(SpritesSelector),
            // Box::new(ObjectSelector),
        ]
        .len());
        Self {
            selectedsprite: 0,
            output: vec![RichText::new("fan-z launched").color(Color32::GREEN)],
            gamewindow: Rc::new(RefCell::new(GameWindow::default())),
            codewindow: Rc::new(RefCell::new(CodeWindow::default())),
            mapwindow: Rc::new(RefCell::new(MapWindow::default())),
            propertieswindow: Rc::new(RefCell::new(PropertiesWindow::default())),
            spriteswindow: Rc::new(RefCell::new(SpritesWindow::default())),
            leftpanel: Rc::new(RefCell::new(vec![
                Box::new(SpritesSelector),
                Box::new(ObjectSelector),
            ])),
            leftpanelselected: 0,
            selectedobject: 0,
            cart: Cart {
                map: Array2D::filled_with(None, 8, 8),
                sprites: vec![],
                objects: vec![],
            },
        }
    }
}

pub trait OptionalWindow {
    fn ui(&mut self, ui: &mut egui::Ui);
}

impl<'a> FanzApp<'a> {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl<'a> eframe::App for FanzApp<'a> {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::style::Visuals::dark());
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Sprite Selector").clicked() {
                        self.leftpanel.borrow_mut().push(Box::new(SpritesSelector))
                    }
                    if ui.button("Object Selector").clicked() {
                        self.leftpanel.borrow_mut().push(Box::new(ObjectSelector))
                    }
                });
                let tmp = &mut self.gamewindow.clone();
                let mut win = tmp.borrow_mut();
                let game = &mut win.game;
                match game {
                    Some(_) => {
                        if ui.button("Stop").clicked() {
                            *game = None;
                        }
                    }
                    None => {
                        if ui.button("Play").clicked() {
                            let state = unsafe {
                                if STATE_PTR.is_null() {
                                    STATE_PTR = Box::leak(Box::new(ZVMState {
                                        buffer: vec![],
                                        keys: vec![],
                                    }));
                                    // cry about it
                                }
                                &mut *STATE_PTR
                            };
                            state.buffer.clear();
                            state.keys.clear();
                            *game = match GameWindow::startgame(&mut self.cart) {
                                Ok(vm) => Some(vm),
                                Err(e) => {
                                    self.output
                                        .push(RichText::new(e.to_string()).color(Color32::RED));
                                    None
                                }
                            };
                            win.enabled = true;
                        }
                    }
                }
            });
        });
        egui::TopBottomPanel::bottom("console_output")
            .resizable(true)
            // .min_height(4.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Console Output");
                });
                egui::ScrollArea::vertical()
                    .max_width(f32::INFINITY)
                    // .max_height(ui.available_height())
                    .show(ui, |ui| {
                        ui.horizontal(|ui| ui.set_min_width(f32::INFINITY));
                        // ui.set_min_width(.0::INFINITY);
                        for t in &self.output {
                            ui.code(t.clone().code());
                        }

                        ui.add_space(ui.available_height());
                    });
            });

        // dbg!(self.leftpanel.borrow_mut().len());
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            let tmp = self.leftpanel.clone();
            let mut tabs = tmp.borrow_mut();
            ui.horizontal(|ui| {
                for (i, tab) in tabs.iter().enumerate() {
                    if ui.button(tab.name()).clicked() {
                        self.leftpanelselected = i;
                    }
                }
            });
            ui.separator();
            if let Some(tab) = tabs.get_mut(self.leftpanelselected) {
                let mut remove = false;
                ui.horizontal(|ui| {
                    ui.heading(tab.name());
                    if ui.button("x").clicked() {
                        remove = true;
                    }
                });
                tab.ui(self, ui);
                if remove {
                    tabs.remove(self.leftpanelselected);
                }
            }
        });

        egui::SidePanel::right("view_panel")
            .resizable(false)
            .show(ctx, |ui| {
                // ui.allocate_space(ui.available_size() * 0.5.0);
                ui.add_space(ui.available_size().x / 4.0);
                ui.vertical_centered(|ui| {
                    toolbtn_ui(
                        ui,
                        "Code",
                        &mut self.codewindow.clone().borrow_mut().enabled,
                    );
                    toolbtn_ui(
                        ui,
                        "Game",
                        &mut self.gamewindow.clone().borrow_mut().enabled,
                    );
                    toolbtn_ui(ui, "Map", &mut self.mapwindow.clone().borrow_mut().enabled);
                    toolbtn_ui(
                        ui,
                        "Sprites",
                        &mut self.spriteswindow.clone().borrow_mut().enabled,
                    );
                });
            });

        let tmp = self.gamewindow.clone();
        let mut win = tmp.borrow_mut();
        if win.enabled {
            egui::Window::new("Game")
                .resize(|r| {
                    r.resizable(false);
                    r.min_size(vec2(160.0, 120.0))
                })
                .show(ctx, |ui| {
                    win.ui(self, ui);
                });
        }
        let tmp = self.codewindow.clone();
        let mut win = tmp.borrow_mut();
        if win.enabled {
            egui::Window::new("Code")
                .resizable(true)
                .resize(|r| r.max_size(ctx.available_rect().size()))
                .show(ctx, |ui| {
                    win.ui(self, ui);
                });
        }
        let tmp = self.mapwindow.clone();
        let mut win = tmp.borrow_mut();
        if win.enabled {
            egui::Window::new("Map Editor")
                .resizable(true)
                .resize(|r| r.max_size(ctx.available_rect().size()))
                .show(ctx, |ui| {
                    win.ui(self, ui);
                });
        }
        let tmp = self.spriteswindow.clone();
        let mut win = tmp.borrow_mut();
        if win.enabled {
            egui::Window::new("Sprite Editor")
                .resizable(true)
                .resize(|r| r.max_size(ctx.available_rect().size()))
                .show(ctx, |ui| {
                    win.ui(self, ui);
                });
        }
    }
}
pub fn draw_sprite(painter: &Painter, rect: Rect, spr: &Sprite) {
    let start = rect.min;
    let size = rect.size();
    let spritesize = vec2(spr.data.num_rows() as f32, spr.data.num_columns() as f32);

    let scale_factor = (size.x / spritesize.x).min(size.y / spritesize.y);
    let offset = vec2(0.0, 0.0);
    // vec2(
    //     (size.x / spritesize.x) - scale_factor,
    //     (size.y / spritesize.y) - scale_factor,
    // ) * scale_factor
    //     / 2.0;
    for x in 0..spr.data.num_rows() {
        for y in 0..spr.data.num_columns() {
            let fillrect = Rect::from_min_size(
                start + offset + vec2(x as f32 * scale_factor, y as f32 * scale_factor),
                vec2(scale_factor, scale_factor),
            );
            painter.rect_filled(fillrect, 0f32, spr.data.get(x, y).unwrap().clone())
        }
    }
}

pub fn toolbtn_ui(ui: &mut egui::Ui, text: &str, on: &mut bool) -> egui::Response {
    // if ui.is_rect_visible(rect)
    let size = ui.available_size().x;
    let (id, rect) = ui.allocate_space(Vec2::new(size / 2.0, size / 2.0));
    let response = ui.interact(rect, id, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut color = visuals.bg_fill;
        if *on {
            color = Color32::BLACK;
        }
        if response.clicked() {
            *on = !*on;
        }
        ui.painter().rect(
            rect,
            5.0,
            color,
            Stroke::new(if response.hovered() { 1.0 } else { 0.0 }, Color32::WHITE),
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            egui::FontId {
                size: 15.0,
                family: egui::FontFamily::Proportional,
            },
            Color32::WHITE,
        );
    }

    response
}
