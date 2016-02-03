use std::error::Error;

use rotor::{Machine, Scope, EarlyScope, Loop, LoopInstance, SpawnError};


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
    /// ```ignore
    /// let sink = loop_inst.add_and_fetch(Fsm::Carbon, |scope| {
    ///     connect_ip(addr, scope)
    /// });
    /// ```
    /// Compare it to *traditional* way:
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
              F: FnOnce(&mut EarlyScope) -> Result<(N, T), E>;
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
    /// ```ignore
    /// let sink = loop_inst.add_and_fetch(Fsm::Carbon, |scope| {
    ///     connect_ip(addr, scope)
    /// });
    /// ```
    /// Compare it to *traditional* way:
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
              F: FnOnce(&mut Scope<M::Context>) -> Result<(N, T), E>;
}

impl<C, M: Machine<Context=C>> LoopExt<M> for Loop<C, M> {
    fn add_and_fetch<F, W, T, E, N>(&mut self, fsm_wrapper: W, fun: F)
        -> Result<T, SpawnError<()>>
        where E: Error + 'static,
              W: FnOnce(N) -> M,
              F: FnOnce(&mut EarlyScope) -> Result<(N, T), E>
    {
        let mut result_opt = None;
        try!(self.add_machine_with(|scope| {
            let (fsm, value) = try!(fun(scope));
            result_opt = Some(value);
            Ok(fsm_wrapper(fsm))
        }));
        Ok(result_opt.unwrap())
    }
}


impl<C, M: Machine<Context=C>> LoopInstanceExt<M> for LoopInstance<C, M> {
    fn add_and_fetch<F, W, T, E, N>(&mut self, fsm_wrapper: W, fun: F)
        -> Result<T, SpawnError<()>>
        where E: Error + 'static,
              W: FnOnce(N) -> M,
              F: FnOnce(&mut Scope<M::Context>) -> Result<(N, T), E>
    {
        let mut result_opt = None;
        try!(self.add_machine_with(|scope| {
            let (fsm, value) = try!(fun(scope));
            result_opt = Some(value);
            Ok(fsm_wrapper(fsm))
        }));
        Ok(result_opt.unwrap())
    }
}
