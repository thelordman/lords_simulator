use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use lords_sim::{Name, Sex, Simulation};

#[wasm_bindgen]
pub struct Runtime {
    sim: Rc<RefCell<Simulation>>,
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(constructor)]
    pub fn new(first: String, middle: String, last: String, sex_str: String) -> Runtime {
        let middle_names = if middle.is_empty() {
            vec![]
        } else {
            middle.split(',').map(str::to_string).collect()
        };

        let sex = match sex_str.as_str() {
            "Female" => Sex::Female,
            _        => Sex::Male,
        };

        let sim = Simulation::new(Name::new(first, middle_names, last), sex);

        Runtime {
            sim: Rc::new(RefCell::new(sim)),
        }
    }

    pub fn start(&self) {
        schedule_tick(Rc::clone(&self.sim));
    }

    pub fn state(&self) -> JsState {
        let state = &self.sim.borrow().state;
        JsState {
            year: state.time.year(),
            month: state.time.month(),
            day: state.time.day(),
            hour: state.time.hour(),
            minute: state.time.minute(),
            seconds: state.time.seconds(),
            second: state.time.second(),
            full_name: state.name.full_name(),
            sex: state.sex.to_string(),
        }
    }
}

fn schedule_tick(sim: Rc<RefCell<Simulation>>) {
    let closure = Closure::once(move || {
        {
            let mut s = sim.borrow_mut();
            s.tick();
        }
        schedule_tick(sim);
    });

    web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            1000, // ms per tick
        )
        .unwrap();

    closure.forget();
}

#[wasm_bindgen]
pub struct JsState {
    pub year: u64,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub seconds: u64,
    pub second: u8,
    full_name: String,
    sex: String,
}

#[wasm_bindgen]
impl JsState {
    #[wasm_bindgen(getter)]
    pub fn full_name(&self) -> String { self.full_name.clone() }
    #[wasm_bindgen(getter)]
    pub fn sex(&self) -> String { self.sex.clone() }
}