extern crate rotor;
extern crate rotor_tools;

use std::time::{Duration};

use rotor::{Loop, Config};
use rotor_tools::timer::interval_func;


fn main() {
    let loop_creator = Loop::new(&Config::new()).unwrap();
    let mut loop_inst = loop_creator.instantiate(());
    loop_inst.add_machine_with(|scope| {
        interval_func(scope, Duration::new(1, 0), |_| {
            println!("Second passed");
        })
    }).unwrap();
    loop_inst.run().unwrap();
}
