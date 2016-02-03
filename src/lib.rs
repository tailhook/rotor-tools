extern crate time;
extern crate void;
extern crate rotor;

pub mod timer;
pub mod loop_ext;
pub use time::{Duration, SteadyTime as Deadline};
