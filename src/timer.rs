//! Time utitities
use std::error::Error;
use std::cmp::max;
use rotor::{Machine, Scope, Response, EventSet, GenericScope};
use void::{Void, unreachable};

use {Duration, Deadline};

/// Ticker state machine
///
/// The structure implements `rotor::Machine` but exposes a simpler protocol
/// that has just one method which is called when timer expires.
///
/// The `Ticker` machine also ensures that there are no spurious events.
pub struct Ticker<M: Timer> {
    deadline: Deadline,
    machine: M,
}

/// Interval state machine
///
/// It's the state machine used for ticker that wakes up with fixed intervals
pub struct Interval<M: SimpleTimer>(Duration, M);

/// A convenience type for declaring state machines
pub type IntervalFunc<C> = Ticker<Interval<Box<FnMut(&mut Scope<C>)>>>;

/// A protocol for the state machine that put into the `Ticker`
pub trait Timer {
    type Context;

    /// Called when time elapsed
    fn timeout(self, scope: &mut Scope<Self::Context>) -> Self;

    /// Calculates the next wakeup time
    ///
    /// `now` -- is time when event processing was started, this is provided
    ///          for performance reasons
    /// `scheduled` -- time when event had to occur
    ///
    /// There are two options to calculate the time. If you just need to
    /// run something on occasion use simply:
    /// ```ignore
    /// now + Duration::seconds(interval)
    /// ```
    ///
    /// Or if you need to run strict number of times and as close as possible
    /// to the multiple of the interval time. You may want:
    /// ```ignore
    /// scheduled + Duration::seconds(interval)
    /// ```
    ///
    /// Note, in both cases mio will run timeout handler on the next tick
    /// of the timer, which means +200 ms by default.
    fn next_wakeup_time(&self, now: Deadline, scheduled: Deadline,
        scope: &mut Scope<Self::Context>)
        -> Deadline;
}

/// The timer trait used in the `Ticker<Interval<T>>`
pub trait SimpleTimer {
    type Context;

    /// Called when time elapsed
    fn timeout(self, scope: &mut Scope<Self::Context>) -> Self;
}

impl<T: Timer> Ticker<T> {
    pub fn new(scope: &mut Scope<T::Context>, machine: T) -> Ticker<T> {
        let now = Deadline::now();
        let next = machine.next_wakeup_time(now, now, scope);
        scope.timeout_ms(
            max(0, (next - now).num_milliseconds()) as u64).unwrap();
        Ticker {
            deadline: next,
            machine: machine,
        }
    }
}

impl<M: Timer> Machine for Ticker<M> {
    type Context = M::Context;
    type Seed = Void;
    fn create(seed: Self::Seed, _scope: &mut Scope<Self::Context>)
        -> Result<Self, Box<Error>>
    {
        unreachable(seed);
    }
    fn ready(self, _events: EventSet, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        // Spurious event
        Response::ok(self)
    }
    fn spawned(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        unreachable!();
    }
    fn timeout(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        let now = Deadline::now();
        if now > self.deadline {
            let newm = self.machine.timeout(scope);
            let next = newm.next_wakeup_time(now, self.deadline, scope);
            scope.timeout_ms(
                max(0, (next - now).num_milliseconds()) as u64).unwrap();
            Response::ok(Ticker {
                deadline: next,
                machine: newm,
            })
        } else {
            Response::ok(self)
        }
    }
    fn wakeup(self, _scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>
    {
        // Spurious wakeup
        Response::ok(self)
    }
}

impl<T: SimpleTimer> Timer for Interval<T> {
    type Context = T::Context;
    fn timeout(self, scope: &mut Scope<Self::Context>) -> Self {
        Interval(self.0, self.1.timeout(scope))
    }
    fn next_wakeup_time(&self, now: Deadline, scheduled: Deadline,
        _scope: &mut Scope<Self::Context>)
        -> Deadline
    {
        // Try to minimize the drift
        let goal = scheduled + self.0;
        if now > goal {
            // But if we are a way too late, just use current time
            return now + self.0;
        } else {
            return goal;
        }
    }
}

impl<C> SimpleTimer for Box<FnMut(&mut Scope<C>)> {
    type Context = C;
    fn timeout(mut self, scope: &mut Scope<Self::Context>) -> Self {
        self(scope);
        self
    }
}

/// A helper function to create intervals from closures
pub fn interval_func<C, F>(scope: &mut Scope<C>, interval: Duration, fun: F)
    -> IntervalFunc<C>
    where F: FnMut(&mut Scope<C>) + 'static
{
    Ticker::new(scope, Interval(interval, Box::new(fun)))
}
