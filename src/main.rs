mod consolebuiltins;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter;
use zsp_core::{
    self,
    runtime::{self, FunctionType, RFunction, ScopeType},
    *,
};

const SCREENWIDTH: usize = 160 * 8;
const SCREENHEIGHT: usize = 120 * 8;
const GAMEWIDTH: usize = 160;
const GAMEHEIGHT: usize = 120;

static mut STATE: *mut GameState = std::ptr::null_mut();

fn main() {
    let contents = std::fs::read_to_string("rom.z")
        .expect("could not read file")
        .chars()
        .filter(|c| c != &'\r')
        .collect::<String>();

    let mut tokens = lexer::lex(contents.clone());

    let libraryfunctions = consolebuiltins::functions();

    let root = match parser::parse(tokens, &contents, &libraryfunctions) {
        Err(e) => {
            println!("{}", e.fmt(&contents));
            panic!()
        }
        Ok(s) => s,
    };
    println!("{:?}", root);

    let mut functions = builtins::functions();

    for (k, v) in libraryfunctions {
        functions.insert(k.clone(), v.clone());
    }
    for fun in root.functions {
        let cfn = fun.1.clone();
        functions.insert(
            fun.0,
            RFunction {
                args: cfn.args.clone(),
                func: FunctionType::InternalFunction(cfn.clone()),
            },
        );
    }

    let scope = Rc::new(RefCell::new(
        root.root.to_scope(ScopeType::Function, HashMap::new()),
    ));
    let mut state = GameState {
        screen: Screen::new(),
        buttons: HashMap::new(),
    };
    unsafe {
        STATE = &mut state;
    }
    runtime::run_root(scope.clone(), &functions, &contents);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("fan-z", SCREENWIDTH as u32, SCREENHEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut i: u8 = 0;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => {
                    if let Some(k) = keycode {
                        state.buttons.insert(k.name(), true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(k) = keycode {
                        state.buttons.insert(k.name(), false);
                    }
                }
                Event::MouseMotion { x, y, .. } => {}
                Event::MouseButtonDown { mouse_btn, .. } => {
                    state.buttons.insert(format!("Mouse{:?}", mouse_btn), true);
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    state.buttons.insert(format!("Mouse{:?}", mouse_btn), false);
                }
                _ => {}
            }
        }

        // scope
        // The rest of the game loop goes here...
        scope.borrow_mut().call_function(
            functions.get("draw").unwrap(),
            vec![],
            &functions,
            &contents,
        );
        // dbg!(&state.screen.data);
        // for (i, c) in state.screen.data.iter().enumerate() {
        //     canvas.set_draw_color(*c);
        //     canvas
        //         .draw_point(sdl2::rect::Point::new(
        //             (i % state.screen.width) as i32,
        //             (i / state.screen.height) as i32,
        //         ))
        //         .unwrap();
        // }

        for x in 0..state.screen.width {
            for y in 0..state.screen.height {
                canvas.set_draw_color(state.screen.get(x, y));
                canvas
                    .fill_rect(sdl2::rect::Rect::new(x as i32 * 8, y as i32 * 8, 8, 8))
                    .unwrap();
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
#[derive(Debug, Clone)]
struct GameState {
    screen: Screen,
    buttons: HashMap<String, bool>,
}

impl GameState {
    fn getbtn(&self, s: String) -> bool {
        if let Some(b) = self.buttons.get(&s) {
            *b
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Screen {
    width: usize,
    height: usize,
    data: Vec<Color>,
}
impl Screen {
    fn new() -> Screen {
        Screen {
            width: GAMEWIDTH,
            height: GAMEHEIGHT,
            data: vec![Color::RGB(0, 0, 0); GAMEWIDTH * GAMEHEIGHT],
        }
    }
    fn set(&mut self, x: usize, y: usize, c: Color) {
        if x > self.width || y > self.height {
            panic!()
        }
        self.data[x + (y * self.width)] = c;
    }
    fn get(&mut self, x: usize, y: usize) -> Color {
        if x > self.width || y > self.height {
            panic!()
        }
        self.data[x + (y * self.width)]
    }
    fn clear(&mut self, c: Color) {
        self.data = vec![c; GAMEWIDTH * GAMEHEIGHT];
    }
}
