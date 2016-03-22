//! Composition tools

use rotor::mio::EventSet;
use rotor::void::{Void, unreachable};
use rotor::{Machine, Scope, Response};


/// Composes two state machines where of the state machines spawns
/// multiple instances of another one
pub enum Spawn<S:Sized, C:Sized> {
    Spawner(S),
    Child(C),
}

impl<X, D, S, C> Machine for Spawn<S, C>
    where S: Machine<Context=X, Seed=D>, C: Machine<Context=X, Seed=D>
{
    type Context = X;
    type Seed = D;

    fn create(seed: D, scope: &mut Scope<X>)
        -> Response<Self, Void>
    {
        C::create(seed, scope).map(Spawn::Child, |x| unreachable(x))
    }
    fn ready(self, events: EventSet, scope: &mut Scope<X>)
        -> Response<Self, Self::Seed>
    {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.ready(events, scope).wrap(Spawner) }
            Child(m) => { m.ready(events, scope).wrap(Child) }
        }
    }
    fn spawned(self, scope: &mut Scope<X>) -> Response<Self, Self::Seed>
    {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.spawned(scope).wrap(Spawner) }
            Child(m) => { m.spawned(scope).wrap(Child) }
        }
    }
    fn timeout(self, scope: &mut Scope<X>) -> Response<Self, Self::Seed> {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.timeout(scope).wrap(Spawner) }
            Child(m) => { m.timeout(scope).wrap(Child) }
        }
    }
    fn wakeup(self, scope: &mut Scope<X>) -> Response<Self, Self::Seed> {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.wakeup(scope).wrap(Spawner) }
            Child(m) => { m.wakeup(scope).wrap(Child) }
        }
    }
}
