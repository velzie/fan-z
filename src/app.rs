#![allow(unused_must_use)]
use crate::{
    codewindow::CodeWindow,
    gamewindow::GameWindow,
    mapwindow::MapWindow,
    spriteswindow::{Sprite, SpritesWindow},
    zvm::{self, ZVMState, STATE_PTR, ZVM},
};
use array2d::Array2D;
use egui::{
    pos2, vec2, Align2, Color32, Id, Key, LayerId, Layout, Painter, Rect, RichText, Sense, Stroke,
    Vec2, Widget,
};
use std::{cell::RefCell, rc::Rc};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FanzApp<'a> {
    pub cart: Cart,

    pub codewindow: Rc<RefCell<CodeWindow>>,

    pub mapwindow: Rc<RefCell<MapWindow>>,
    pub spriteswindow: Rc<RefCell<SpritesWindow>>,

    pub selectedsprite: usize,
    #[serde(skip)]
    pub gamewindow: Rc<RefCell<GameWindow<'a>>>,

    #[serde(skip)]
    pub output: Vec<RichText>,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Cart {
    pub code: String,
    pub sprites: Vec<Sprite>,
    pub map: Array2D<Option<usize>>,
}

impl<'a> Default for FanzApp<'a> {
    fn default() -> Self {
        Self {
            selectedsprite: 0,
            output: vec![RichText::new("fan-z launched").color(Color32::GREEN)],
            gamewindow: Rc::new(RefCell::new(GameWindow::default())),
            codewindow: Rc::new(RefCell::new(CodeWindow::default())),
            mapwindow: Rc::new(RefCell::new(MapWindow::default())),
            spriteswindow: Rc::new(RefCell::new(SpritesWindow::default())),
            cart: Cart {
                map: Array2D::filled_with(None, 8, 8),
                code: "".into(),
                sprites: vec![],
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
                let tmp = &mut self.gamewindow.clone();
                let mut win = tmp.borrow_mut();
                let vm = &mut win.vm;
                match vm {
                    Some(_) => {
                        if ui.button("Stop").clicked() {
                            *vm = None;
                            unsafe {
                                drop(STATE_PTR);
                            }
                        }
                    }
                    None => {
                        unsafe {
                            STATE_PTR = Box::leak(Box::new(ZVMState {
                                buffer: vec![],
                                keys: vec![],
                            }))
                        };
                        if ui.button("Play").clicked() {
                            *vm = match ZVM::start(&self.cart.code) {
                                Ok(vm) => Some(vm),
                                Err(e) => {
                                    let o = zvm::errfmt(e, &self.cart.code);
                                    self.output
                                        .push(RichText::new(o.to_string()).color(Color32::RED));
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

        if self.mapwindow.clone().borrow_mut().enabled
            || self.spriteswindow.clone().borrow_mut().enabled
        {
            egui::SidePanel::left("sprite_selector").show(ctx, |ui| {
                // ui.set_max_width(ui.available_width());
                ui.heading("Sprite Selector");
                ui.horizontal_wrapped(|ui| {
                    for (i, sprite) in self.cart.sprites.iter().enumerate() {
                        let (id, rect) = ui.allocate_space(Vec2::new(32.0, 32.0));
                        let response = ui.interact(rect, id, egui::Sense::click());

                        ui.painter().rect(
                            rect,
                            0f32,
                            Color32::BLACK,
                            if i == self.selectedsprite {
                                Stroke::new(2.0, Color32::WHITE)
                            } else {
                                Stroke::none()
                            },
                        );
                        for x in 0..8 {
                            for y in 0..8 {
                                let fillrect = Rect::from_min_size(
                                    rect.min + vec2(x as f32 * 4.0, y as f32 * 4.0),
                                    vec2(4.0, 4.0),
                                );
                                ui.painter().rect_filled(
                                    fillrect,
                                    0f32,
                                    sprite.data.get(x, y).unwrap().clone(),
                                )
                            }
                        }
                        if response.clicked() {
                            self.selectedsprite = i;
                        }
                    }
                });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!("Selected Sprite: {}", self.selectedsprite))
                });
            });
        }
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
