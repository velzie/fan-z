use std::collections::HashMap;
// use egui:
// use zsp_core:
use crate::zvm::{ZEvent, STATE_PTR};
use egui::Color32;
use zsp_core::{
    exceptions::Exception,
    func,
    runtime::{downcast_dyn, DynObject, DynObjectContainer, FunctionType, RFunction, Value},
};
pub fn functions() -> HashMap<String, RFunction> {
    HashMap::from([
        func!("key", keypressed, 1),
        func!("put", put, 1),
        func!("color", color, 3),
        func!("gset", gset, 3),
        func!("rect", rect, 5),
        // func!("clear", clear, 1),
    ])
}
#[derive(Debug, Clone)]
pub struct ZColor {
    r: u8,
    g: u8,
    b: u8,
}
impl<'a> DynObject<'a> for ZColor {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ZColor>")
    }
    fn fields(&self) -> HashMap<String, std::rc::Rc<std::cell::RefCell<Value<'a>>>> {
        HashMap::new()
    }
}
impl ZColor {
    pub fn tocolor(&self) -> Color32 {
        Color32::from_rgb(self.r, self.g, self.b)
    }
}
fn color<'a>(inp: Vec<Value<'a>>) -> Result<Value<'_>, Exception> {
    Ok(Value::DynObject(DynObjectContainer {
        val: Box::new(ZColor {
            r: inp[0].to_number() as u8,
            g: inp[1].to_number() as u8,
            b: inp[2].to_number() as u8,
        }),
    }))
}

fn keypressed<'a>(inp: Vec<Value<'a>>) -> Result<Value<'_>, Exception> {
    Ok(Value::Bool(unsafe {
        (*STATE_PTR).keys.contains(&inp[0].to_string())
    }))
}
fn gset<'a>(mut inp: Vec<Value<'a>>) -> Result<Value<'_>, Exception> {
    unsafe {
        (*STATE_PTR).buffer.push(ZEvent::GSet {
            color: downcast_dyn::<ZColor>(
                &mut inp[2].as_ref().clone().borrow_mut().as_dyn_object(),
            ).tocolor(),
            x: inp[0].to_number(),
            y: inp[1].to_number(),
        });
    }
    Ok(Value::Null)
}
fn rect<'a>(mut inp: Vec<Value<'a>>) -> Result<Value<'_>, Exception> {
    unsafe {
        (*STATE_PTR).buffer.push(ZEvent::Rect {
            color: downcast_dyn::<ZColor>(
                &mut inp[4].as_ref().clone().borrow_mut().as_dyn_object(),
            ).tocolor(),
            x: inp[0].to_number(),
            y: inp[1].to_number(),
            w: inp[2].to_number(),
            h: inp[3].to_number(),
        });
    }
    Ok(Value::Null)
}

fn put<'a>(inp: Vec<Value<'a>>) -> Result<Value<'_>, Exception> {
    unsafe {
        (*STATE_PTR).buffer.push(ZEvent::Put(inp[0].to_string()));
    }
    Ok(Value::Null)
}
