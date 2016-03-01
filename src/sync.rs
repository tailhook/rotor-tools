//! Provides state machine that is guarded by Mutex
//!
//! This allows the machine to be manipulated from multiple threads. But you
//! must be careful to not to break the state machine
use std::mem;
use std::sync::{Arc, Mutex};

use rotor::{Machine, EventSet, Scope, Response};
use rotor::{Void};

pub struct Mutexed<M>(Arc<Mutex<M>>);

/// A trait which allows to replace the state machine with dummy/null/None
///
/// Since we put machine (used with `Mutexed`) into a `Arc<Mutex<M>>` we need
/// to replace it with some other value while the real value is moved off the
/// state machine to be able to replace it by code, having self by-value.
///
/// When the thread working with the state machine panics while machine is
/// moved off the place, the poisoned lock is left with `Replaceable::empty`
/// value. When lock is poisoned we unwrap the value and use it as a valid
/// state machine. So you can continue to work after panic with the clean
/// state after a crash (perhaps if the panic was in different thread).
///
/// The text above means you should be able to "restart" the state machine
/// from the `empty()` state.
trait Replaceable: Machine {
    /// Return the empty value that may be used as replacement
    ///
    /// **The method must be cheap to compute**. Because it's executed on
    /// every action of a state machine.
    fn empty() -> Self;
    /// Restart a state machine from `empty()` state
    ///
    /// This method is called before calling any other action methods when
    /// lock holding the state machine was poisoned.
    ///
    /// Note that after the `restart` current event is discarded, it's assumed
    /// that state machine is already arranged to receive some new events
    /// (i.e. it's useless to keep old `ready()` event if new connection is
    /// just being established)
    ///
    /// While you can check the state of the old machine (a `self`), and even
    /// return it as is, it's strongly discouraged, as you can't know exact
    /// kind of failure that happened in other thread (when lock was
    /// poisoned). But in case protocol is super-simple (like line-based
    /// without exceptions) and it's not security critical (i.e. monitoring
    /// using graphite), you may reuse old state machine or parts there of.
    ///
    /// Default implementation is just to panic
    fn restart(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        panic!("State machine has been poisoned");
    }
}

#[inline]
fn locked_call<M, F>(scope: &mut Scope<M::Context>, me: Mutexed<M>,
    fun: F)
    -> Response<Mutexed<M>, M::Seed>
    where M: Replaceable,
          F: FnOnce(M, &mut Scope<M::Context>) -> Response<M, M::Seed>
{
    let fake_result = match me.0.lock() {
        Ok(mut guard) => {
            let fsm = mem::replace(&mut *guard, Replaceable::empty());
            let res = fun(fsm, scope);
            res.wrap(|new_machine| {
                // thows off an `empty()` instance
                mem::replace(&mut *guard, new_machine);
                ()
            })
        }
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            let fsm = mem::replace(&mut *guard, Replaceable::empty());
            let res = fsm.restart(scope);
            res.wrap(|new_machine| {
                // thows off an `empty()` instance
                mem::replace(&mut *guard, new_machine);
                ()
            })
        }
    };
    fake_result.wrap(|()| me)
}

impl<M: Replaceable> Machine for Mutexed<M> {
    type Context = M::Context;
    type Seed = M::Seed;
    fn create(seed: Self::Seed, scope: &mut Scope<M::Context>)
        -> Response<Self, Void>
    {
        M::create(seed, scope).wrap(Mutex::new).wrap(Arc::new).wrap(Mutexed)
    }
    fn ready(self, events: EventSet, scope: &mut Scope<M::Context>)
        -> Response<Self, Self::Seed>
    {
        locked_call(scope, self, |fsm, scope| fsm.ready(events, scope))
    }
    fn spawned(self, scope: &mut Scope<M::Context>)
        -> Response<Self, Self::Seed>
    {
        locked_call(scope, self, |fsm, scope| fsm.spawned(scope))
    }
    fn timeout(self, scope: &mut Scope<M::Context>)
        -> Response<Self, Self::Seed>
    {
        locked_call(scope, self, |fsm, scope| fsm.timeout(scope))
    }
    fn wakeup(self, scope: &mut Scope<M::Context>)
        -> Response<Self, Self::Seed>
    {
        locked_call(scope, self, |fsm, scope| fsm.wakeup(scope))
    }
}
