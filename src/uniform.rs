//! Uniform state machine abstraction
//!
//! By "uniform" we mean a state machine which does the same action on any
//! event handler. I.e. it can check some global state on every wakeup,
//! timeout, or other event and react the same way.

use rotor::{Scope, Machine, Response, EventSet, Void};

struct Uniform<T: Action>(T);

trait Action: Sized {
    type Context;
    type Seed;
    fn create(seed: Self::Seed, scope: &mut Scope<Self::Context>)
        -> Response<Self, Void>;
    fn action(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>;
}

impl<T: Action> Machine for Uniform<T> {
    type Context = T::Context;
    type Seed = T::Seed;
    fn create(seed: Self::Seed, scope: &mut Scope<Self::Context>)
        -> Response<Self, Void>
    {
        Action::create(seed, scope).wrap(Uniform)
    }
    fn ready(self, _events: EventSet, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        self.0.action(scope).wrap(Uniform)
    }
    fn spawned(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        self.0.action(scope).wrap(Uniform)
    }
    fn timeout(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        self.0.action(scope).wrap(Uniform)
    }
    fn wakeup(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        self.0.action(scope).wrap(Uniform)
    }
}
