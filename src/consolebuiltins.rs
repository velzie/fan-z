use std::{collections::HashMap, fmt::write};

use sdl2::pixels::Color;
use zsp_core::{
    exceptions::Exception,
    func,
    runtime::{downcast_dyn, DynObject, DynObjectContainer, FunctionType, RFunction, Value},
};

use crate::STATE;

pub fn functions() -> HashMap<String, RFunction> {
    HashMap::from([
        func!("gset", gset, 3),
        func!("color", color, 3),
        func!("key", key, 1),
        func!("clear", clear, 1),
        func!("rect", rect, 5),
    ])
}
fn gset(inp: Vec<Value>) -> Result<Value, Exception> {
    unsafe {
        (*STATE).screen.set(
            inp[0].to_number() as usize,
            inp[1].to_number() as usize,
            downcast_dyn::<ZColor>(&mut inp[2].clone().as_ref().borrow_mut().as_dyn_object())
                .tocolor(),
        );
    }
    Ok(Value::Null)
}
fn clear(inp: Vec<Value>) -> Result<Value, Exception> {
    unsafe {
        (*STATE).screen.clear(
            downcast_dyn::<ZColor>(&mut inp[0].clone().as_ref().borrow_mut().as_dyn_object())
                .tocolor(),
        )
    }
    Ok(Value::Null)
}

fn rect(inp: Vec<Value>) -> Result<Value, Exception> {
    unsafe {
        let c = downcast_dyn::<ZColor>(&mut inp[4].clone().as_ref().borrow_mut().as_dyn_object())
            .tocolor();
        for x in inp[0].to_number() as usize..inp[2].to_number() as usize {
            for y in inp[1].to_number() as usize..inp[3].to_number() as usize {
                (*STATE).screen.set(x, y, c);
            }
        }
    }
    Ok(Value::Null)
}
fn key(inp: Vec<Value>) -> Result<Value, Exception> {
    Ok(Value::Bool(unsafe { (*STATE).getbtn(inp[0].to_string()) }))
}
fn color(inp: Vec<Value>) -> Result<Value, Exception> {
    Ok(Value::DynObject(DynObjectContainer {
        val: Box::new(ZColor {
            r: inp[0].to_number() as u8,
            g: inp[1].to_number() as u8,
            b: inp[2].to_number() as u8,
        }),
    }))
}

#[derive(Debug, Clone)]
struct ZColor {
    r: u8,
    g: u8,
    b: u8,
}
impl ZColor {
    fn tocolor(&self) -> Color {
        Color::RGB(self.r, self.g, self.b)
    }
}
impl<'a> DynObject<'a> for ZColor {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<zcolor>")
    }
    fn fields(&self) -> HashMap<String, std::rc::Rc<std::cell::RefCell<Value<'a>>>> {
        HashMap::new()
    }
}
