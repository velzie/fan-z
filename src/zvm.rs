use crate::consolebuiltins::{self, ZColor};
use egui::{Color32, Pos2, Rect};
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use zsp_core::{
    builtins,
    exceptions::Exception,
    lexer, parser,
    runtime::{self, FunctionType, RFunction, Scope, ScopeType},
};

// raw pointers
pub static mut STATE_PTR: *mut ZVMState = std::ptr::null_mut();

pub enum ZEvent {
    GSet {
        x: f32,
        y: f32,
        color: Color32,
    },
    Rect {
        x: f32,
        y: f32,
        h: f32,
        w: f32,
        color: Color32,
    },
    Sprite {
        x: f32,
        y: f32,
        sprite: usize,
    },
    Put(String),
}
pub struct ZVM<'a> {
    pub contents: String,
    pub functions: HashMap<String, RFunction>,
    pub root_scope: Rc<RefCell<Scope<'a>>>,
}
pub struct ZVMState {
    pub buffer: Vec<ZEvent>,
    pub keys: Vec<String>,
}

impl<'a> ZVM<'a> {
    pub fn start(contents: &String) -> Result<ZVM<'a>, Exception> {
        match std::panic::catch_unwind(|| -> Result<ZVM<'a>, Exception> {
            let tokens = lexer::lex(contents.clone());

            let libraryfunctions = consolebuiltins::functions();

            let root = parser::parse(tokens, &contents, &libraryfunctions)?;
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
            runtime::run_root(scope.clone(), &functions, &contents)?;

            Ok(ZVM {
                functions,
                root_scope: scope,
                contents: contents.clone(),
            })
        }) {
            Ok(o) => o,
            Err(_) => Err(Exception::new(
                2,
                "InternalPanicException",
                &format!("Internal panic. check console for error and report bug"),
            )),
        }
    }

    pub fn draw(&mut self) -> Result<(), Exception> {
        self.root_scope.borrow_mut().call_function(
            self.functions.get("draw").unwrap(),
            vec![],
            &self.functions,
            &self.contents,
        )?;
        Ok(())
    }
}

pub fn errfmt(exception: Exception, input: &String) -> String {
    let mut i = 0;
    let mut lines = 0;
    let mut offset = 0;
    while i < exception.idx {
        if input.chars().nth(i).unwrap() == '\n' {
            lines += 1;
            offset = 0;
        }
        offset += 1;
        i += 1;
    }
    let allines: Vec<&str> = input.lines().collect();

    let line1 = format!(
        "      \"{}\"     {}",
        allines[lines],
        format!("at line {}, col {}", lines.to_string(), offset.to_string())
    );
    let line2 = format!(
        "      {}{}      {}",
        " ".repeat(offset - 1),
        "^",
        exception.errtype
    );
    let line3 = format!("{} {}", "ERROR:", exception.message);

    let dasheslen = line3.len() / 2;
    // dbg!(dasheslen);

    format!(
        "{}\n{}\n{}\n{}\n{}",
        "-".repeat(dasheslen),
        line1,
        line2,
        line3,
        "-".repeat(dasheslen)
    )
}
