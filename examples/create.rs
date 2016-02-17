#[macro_use(rotor_compose)] extern crate rotor;
extern crate rotor_tools;

use std::time::{Duration};

use rotor::{Loop, Config, Scope, Response, Void};
use rotor::void::unreachable;
use rotor_tools::timer::{IntervalFunc, interval_func};
use rotor_tools::loop_ext::LoopInstanceExt;

#[derive(PartialEq, Eq)]
struct Dummy;

struct Context;

rotor_compose!(enum Fsm/Seed<Context> {
    Timer(IntervalFunc<Context>),
});

/// Some imaginary API that returns a pair fo things
fn create_pair<C>(scope: &mut Scope<C>)
    -> Response<(IntervalFunc<C>, Dummy), Void>
{
    interval_func(scope, Duration::new(1, 0), |_| {
        println!("Second passed");
    }).wrap(|fsm| (fsm, Dummy))
}

fn main() {
    let loop_creator = Loop::new(&Config::new()).unwrap();
    let mut loop_inst = loop_creator.instantiate(Context);
    let value: Dummy = loop_inst.add_and_fetch(Fsm::Timer, |scope| {
        create_pair(scope)
    }).unwrap();
    assert!(value == Dummy);
    loop_inst.run().unwrap();
}
