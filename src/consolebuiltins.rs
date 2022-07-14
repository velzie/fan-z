use std::{collections::HashMap, fmt::write};

use sdl2::pixels::Color;
use zsp_core::{
    func,
    runtime::{DynObject, FunctionType, RFunction, Value},
};

use crate::STATE;

pub fn functions() -> HashMap<String, RFunction> {
    HashMap::from([func!("gset", gset, 3)])
}
fn gset(inp: Vec<Value>) -> Value {
    unsafe {
        let rv = inp[2].as_dyn_object().val;
        (*STATE).screen.set(
            inp[0].to_number() as usize,
            inp[1].to_number() as usize,
            dyncast(&mut rv).tocolor(),
        );
    }
    Value::Null
}

fn dyncast(v: &mut Box<dyn DynObject>) -> ZColor {
    let mut mutv: *mut dyn DynObject = &mut **v;
    // let mut inptr = (&mut (*v) as *mut dyn DynObject);

    ZColor { r: 0, g: 0, b: 0 }
    // unsafe { &mut *(*( as *mut ZColor) }
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
