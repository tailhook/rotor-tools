//! The traits which make main loop construction nicer
use std::error::Error;

use rotor::{Machine, Scope, EarlyScope, Loop, LoopInstance, SpawnError};
use rotor::{Response, Void};


/// Convenience enhancements to the main loop creator
pub trait LoopExt<M> {
    /// This method is useful for things that have a state machine and an
    /// accessor to it.
    ///
    /// Function looks a little bit complicated with generic params. But it
    /// solves very useful pattern in easier way.
    ///
    /// Examples:
    ///
    /// * `rotor_dns::create_resolve()`
    /// * `rotor_carbon::connect_ip()`
    ///
    /// Usage is simple (carbon example):
    ///
    /// ```ignore
    /// let sink = loop_inst.add_and_fetch(Fsm::Carbon, |scope| {
    ///     connect_ip(addr, scope)
    /// });
    /// ```
    ///
    /// Compare it to *traditional* way:
    ///
    /// ```ignore
    /// let mut sink_opt = None;
    /// loop_creator.add_machine_with(|scope| {
    ///     let (fsm, sink) = connect_ip(addr, scope).unwrap();
    ///     sink_opt = Some(sink);
    ///     Ok(Fsm::Carbon(fsm))
    /// }).unwrap();
    /// let sink = sink_opt.unwrap();
    /// ```
    ///
    fn add_and_fetch<F, W, T, E, N>(&mut self, fsm_wrapper: W, fun: F)
        -> Result<T, SpawnError<()>>
        where E: Error + 'static,
              W: FnOnce(N) -> M,
              F: FnOnce(&mut EarlyScope) -> Response<(N, T), Void>;
}

/// Convenience enhancements to the main loop creator instance
pub trait LoopInstanceExt<M: Machine> {
    /// This method is useful for things that have a state machine and an
    /// accessor to it.
    ///
    /// Function looks a little bit complicated with generic params. But it
    /// solves very useful pattern in easier way.
    ///
    /// Examples:
    ///
    /// * `rotor_dns::create_resolve()`
    /// * `rotor_carbon::connect_ip()`
    ///
    /// Usage is simple (carbon example):
    ///
    /// ```ignore
    /// let sink = loop_inst.add_and_fetch(Fsm::Carbon, |scope| {
    ///     connect_ip(addr, scope)
    /// });
    /// ```
    ///
    /// Compare it to *traditional* way:
    ///
    /// ```ignore
    /// let mut sink_opt = None;
    /// loop_creator.add_machine_with(|scope| {
    ///     let (fsm, sink) = connect_ip(addr, scope).unwrap();
    ///     sink_opt = Some(sink);
    ///     Ok(Fsm::Carbon(fsm))
    /// }).unwrap();
    /// let sink = sink_opt.unwrap();
    /// ```
    ///
    fn add_and_fetch<F, W, T, E, N>(&mut self, fsm_wrapper: W, fun: F)
        -> Result<T, SpawnError<()>>
        where E: Error + 'static,
              W: FnOnce(N) -> M,
              F: FnOnce(&mut Scope<M::Context>) -> Response<(N, T), Void>;
}

impl<M: Machine> LoopExt<M> for Loop<M> {
    fn add_and_fetch<F, W, T, E, N>(&mut self, fsm_wrapper: W, fun: F)
        -> Result<T, SpawnError<()>>
        where E: Error + 'static,
              W: FnOnce(N) -> M,
              F: FnOnce(&mut EarlyScope) -> Response<(N, T), Void>
    {
        let mut result_opt = None;
        try!(self.add_machine_with(|scope| {
            fun(scope).wrap(|(fsm, value)| {
                result_opt = Some(value);
                fsm_wrapper(fsm)
            })
        }));
        Ok(result_opt.unwrap())
    }
}


impl<M: Machine> LoopInstanceExt<M> for LoopInstance<M> {
    fn add_and_fetch<F, W, T, E, N>(&mut self, fsm_wrapper: W, fun: F)
        -> Result<T, SpawnError<()>>
        where E: Error + 'static,
              W: FnOnce(N) -> M,
              F: FnOnce(&mut Scope<M::Context>) -> Response<(N, T), Void>
    {
        let mut result_opt = None;
        try!(self.add_machine_with(|scope| {
            fun(scope).wrap(|(fsm, value)| {
                result_opt = Some(value);
                fsm_wrapper(fsm)
            })
        }));
        Ok(result_opt.unwrap())
    }
}
