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

fn main() {
    let contents = std::fs::read_to_string("rom.z")
        .expect("could not read file")
        .chars()
        .filter(|c| c != &'\r')
        .collect::<String>();

    let mut tokens = lexer::lex(contents.clone());

    let libraryfunctions = HashMap::new();

    let root = parser::parse(tokens, &contents, &libraryfunctions);
    println!("{:?}", root);

    let mut functions = builtins::functions();

    // for (k, v) in libraryfunctions {
    //     functions.insert(k.clone(), v.clone());
    // }
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
    runtime::run_root(scope.clone(), &functions, &contents);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 128, 128)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
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
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
