//! Time utitities
use std::time::Duration;
use rotor::{Machine, Scope, Response, EventSet, GenericScope, Time};
use rotor::void::{Void, unreachable};

/// Ticker state machine
///
/// The structure implements `rotor::Machine` but exposes a simpler protocol
/// that has just one method which is called when timer expires.
///
/// The `Ticker` machine also ensures that there are no spurious events.
pub struct Ticker<M: Timer> {
    deadline: Time,
    machine: M,
}

/// Interval state machine
///
/// It's the state machine used for ticker that wakes up with fixed intervals
pub struct Interval<M: SimpleTimer>(Duration, M);

/// A convenience type for declaring state machines
pub type IntervalFunc<C> = Ticker<Interval<Box<FnMut(&mut Scope<C>) + Send>>>;

/// A protocol for the state machine that put into the `Ticker`
pub trait Timer {
    type Context;

    /// Called when time elapsed
    fn timeout(self, scope: &mut Scope<Self::Context>) -> Self;

    /// Calculates the next wakeup time
    ///
    /// `scheduled` -- time when event had to occur
    ///
    /// There are two options to calculate the time. If you just need to
    /// run something on occasion use simply:
    /// ```ignore
    /// scope.now() + Duration::new(interval, )
    /// ```
    ///
    /// Or if you need to run strict number of times and as close as possible
    /// to the multiple of the interval time. You may want:
    /// ```ignore
    /// scheduled + Duration::new(interval, 0)
    /// ```
    ///
    /// Note, in both cases mio will run timeout handler on the next tick
    /// of the timer, which means +200 ms by default.
    fn next_wakeup_time(&self, scheduled: Time,
        scope: &mut Scope<Self::Context>)
        -> Time;
}

/// The timer trait used in the `Ticker<Interval<T>>`
pub trait SimpleTimer {
    type Context;

    /// Called when time elapsed
    fn timeout(self, scope: &mut Scope<Self::Context>) -> Self;
}

impl<T: Timer> Ticker<T> {
    pub fn new(scope: &mut Scope<T::Context>, machine: T)
        -> Response<Ticker<T>, Void>
    {
        let next = machine.next_wakeup_time(scope.now(), scope);
        Response::ok(Ticker {
            deadline: next,
            machine: machine,
        }).deadline(next)
    }
}

impl<M: Timer> Machine for Ticker<M> {
    type Context = M::Context;
    type Seed = Void;
    fn create(seed: Self::Seed, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Void>
    {
        unreachable(seed);
    }
    fn ready(self, _events: EventSet, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        // Spurious event
        let deadline = self.deadline;
        Response::ok(self).deadline(deadline)
    }
    fn spawned(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        unreachable!();
    }
    fn timeout(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        let now = scope.now();
        if now >= self.deadline {
            let newm = self.machine.timeout(scope);
            let next = newm.next_wakeup_time(self.deadline, scope);
            Response::ok(Ticker {
                deadline: next,
                machine: newm,
            }).deadline(next)
        } else {
            // Spurious timeout
            // TODO(tailhook) should not happen when we get rid of
            // scope.timeout_ms()
            let deadline = self.deadline;
            Response::ok(self).deadline(deadline)
        }
    }
    fn wakeup(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        // Spurious wakeup
        let deadline = self.deadline;
        Response::ok(self).deadline(deadline)
    }
}

impl<T: SimpleTimer> Timer for Interval<T> {
    type Context = T::Context;
    fn timeout(self, scope: &mut Scope<Self::Context>) -> Self {
        Interval(self.0, self.1.timeout(scope))
    }
    fn next_wakeup_time(&self, scheduled: Time,
        scope: &mut Scope<Self::Context>)
        -> Time
    {
        // Try to minimize the drift
        let goal = scheduled + self.0;
        if scope.now() > goal {
            // But if we are a way too late, just use current time
            return scope.now() + self.0;
        } else {
            return goal;
        }
    }
}

impl<C> SimpleTimer for Box<FnMut(&mut Scope<C>) + Send> {
    type Context = C;
    fn timeout(mut self, scope: &mut Scope<Self::Context>) -> Self {
        self(scope);
        self
    }
}

/// A helper function to create intervals from closures
pub fn interval_func<C, F>(scope: &mut Scope<C>, interval: Duration, fun: F)
    -> Response<IntervalFunc<C>, Void>
    where F: FnMut(&mut Scope<C>) + 'static + Send
{
    Ticker::new(scope, Interval(interval, Box::new(fun)))
}
