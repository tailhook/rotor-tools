//! Composition tools

use rotor::mio::EventSet;
use rotor::void::{Void, unreachable};
use rotor::{Machine, Scope, Response};


/// Composes two state machines where of the state machines spawns
/// multiple instances of another one
pub enum Spawn<S: Spawner> {
    Spawner(S),
    Child(S::Child),
}

pub trait Spawner {
    type Child: Machine<Seed=Void>;
    type Seed;

    fn spawn(seed: Self::Seed,
        scope: &mut Scope<<Self::Child as Machine>::Context>)
        -> Response<Self::Child, Void>;
}

impl<T, C, S> Spawner for ::uniform::Uniform<T>
    where T: Spawner<Seed=S> + ::uniform::Action<Seed=S, Context=C>
{
    type Child = T::Child;
    type Seed = <T as Spawner>::Seed;

    fn spawn(seed: <Self as Spawner>::Seed,
        scope: &mut Scope<<<Self as Spawner>::Child as Machine>::Context>)
        -> Response<Self::Child, Void>
    {
        T::spawn(seed, scope)
    }
}

impl<S, C, D> Machine for Spawn<S>
    where S: Spawner<Child=C, Seed=D> + Machine<Context=C::Context, Seed=D>,
          C: Machine<Seed=Void>,
{
    type Context = <S::Child as Machine>::Context;
    type Seed = <S as Spawner>::Seed;

    fn create(seed: <S as Spawner>::Seed, scope: &mut Scope<Self::Context>)
        -> Response<Self, Void>
    {
        S::spawn(seed, scope).wrap(Spawn::Child)
    }
    fn ready(self, events: EventSet, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.ready(events, scope).wrap(Spawner) }
            Child(m) => { m.ready(events, scope)
                           .map(Child, |x| unreachable(x)) }
        }
    }
    fn spawned(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.spawned(scope).wrap(Spawner) }
            Child(m) => { m.spawned(scope).map(Child, |x| unreachable(x)) }
        }
    }
    fn timeout(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.timeout(scope).wrap(Spawner) }
            Child(m) => { m.timeout(scope).map(Child, |x| unreachable(x)) }
        }
    }
    fn wakeup(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        use self::Spawn::*;
        match self {
            Spawner(m) => { m.wakeup(scope).wrap(Spawner) }
            Child(m) => { m.wakeup(scope).map(Child, |x| unreachable(x)) }
        }
    }
}
