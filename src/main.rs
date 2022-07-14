mod consolebuiltins;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};
use zsp_core::{
    self,
    runtime::{self, FunctionType, RFunction, ScopeType},
    *,
};
const WIDTH: usize = 160 / 2;
const HEIGHT: usize = 120 / 2;

static mut STATE: *mut GameState = std::ptr::null_mut();
fn main() {
    let contents = std::fs::read_to_string("rom.z")
        .expect("could not read file")
        .chars()
        .filter(|c| c != &'\r')
        .collect::<String>();

    let mut tokens = lexer::lex(contents.clone());

    let libraryfunctions = consolebuiltins::functions();

    let root = parser::parse(tokens, &contents, &libraryfunctions);
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
    };
    unsafe {
        STATE = &mut state;
    }
    runtime::run_root(scope.clone(), &functions, &contents);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
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
        for x in 0..state.screen.width {
            for y in 0..state.screen.height {
                canvas.set_draw_color(state.screen.get(x, y));
                canvas
                    .draw_point(sdl2::rect::Point::new(x as i32, y as i32))
                    .unwrap();
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 * 2));
    }
}
#[derive(Debug, Clone)]
struct GameState {
    screen: Screen,
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
            width: WIDTH,
            height: HEIGHT,
            data: vec![Color::RGB(0, 0, 0); WIDTH * HEIGHT],
        }
    }
    fn set(&mut self, x: usize, y: usize, c: Color) {
        if x > self.width || y > self.height {
            panic!()
        }
        self.data[x + y * self.height] = c;
    }
    fn get(&mut self, x: usize, y: usize) -> Color {
        if x > self.width || y > self.height {
            panic!()
        }
        self.data[x + y * self.height]
    }
}
