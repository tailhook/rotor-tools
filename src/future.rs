use std::fmt;
use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

use rotor::{Notifier, GenericScope};

trait ReadFuture<T> {
    fn ready(&self) -> bool;
    fn take(&mut self) -> Option<T>;
    /// Only for debug trait
    fn peek(&self) -> Option<&T>;
}

pub trait MakeFuture<T> {
    fn make_future(self) -> Future<T>;
}

pub trait GetNotifier {
    fn get_notifier(self) -> Notifier;
}

impl<'a, T: GenericScope> GetNotifier for &'a mut T {
    fn get_notifier(self) -> Notifier {
        self.notifier()
    }
}
impl GetNotifier for Notifier {
    fn get_notifier(self) -> Notifier {
        return self;
    }
}

pub struct Future<T>(Arc<Mutex<ReadFuture<T>>>);

pub struct FutureImpl<I, O, F: FnOnce(I) -> O>{
    output: Option<O>,
    convert: Option<F>,
    notifier: Notifier,
    phantom: PhantomData<*const I>,
}

impl<I, O, F> MakeFuture<O> for Arc<Mutex<FutureImpl<I, O, F>>>
    where I: 'static, O: 'static, F: FnOnce(I) -> O + 'static
{
    fn make_future(self) -> Future<O> {
        Future(self)
    }
}

impl<I, O, F:FnOnce(I) -> O> FutureImpl<I, O, F> {
    pub fn new(fun: F, notify: Notifier) -> FutureImpl<I, O, F> {
        FutureImpl {
            output: None,
            convert: Some(fun),
            notifier: notify,
            phantom: PhantomData,
        }
    }
    pub fn put(&mut self, t: O) {
        self.output = Some(t);
        self.notifier.wakeup().expect("wakeup of state machine");
    }
    pub fn convert(&mut self) -> F {
        self.convert.take().unwrap()
    }
}

impl<I, O, F:FnOnce(I) -> O> ReadFuture<O> for FutureImpl<I, O, F> {
    fn ready(&self) -> bool {
        self.output.is_some()
    }
    fn take(&mut self) -> Option<O> {
        self.output.take()
    }
    fn peek(&self) -> Option<&O> {
        self.output.as_ref()
    }
}

impl<T: Sized> Future<T> {
    pub fn is_done(&self) -> bool {
        self.0.lock().expect("future can be locked").ready()
    }
    pub fn consume(self) -> Result<T, Self> {
        match self.0.lock().expect("future can be locked").take() {
            Some(x) => return Ok(x),
            None => {}
        }
        Err(self)
    }
}

pub fn new<I, O, F, N>(notifier: N, fun: F)
    -> Arc<Mutex<FutureImpl<I, O, F>>>
    where O: 'static, F: FnOnce(I) -> O + 'static, N: GetNotifier
{
    Arc::new(Mutex::new(FutureImpl::new(fun, notifier.get_notifier())))
}

impl<T: fmt::Debug> fmt::Debug for Future<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0.lock() {
            Ok(locked) => match locked.peek() {
                Some(value) => write!(fmt, "Future({:?})", value),
                None => write!(fmt, "Future(<Waiting>)"),
            },
            Err(_) => write!(fmt, "Future(<Poisoned>)"),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate rotor_test;
    use std::sync::{Arc, Mutex};

    use super::{new, FutureImpl, Future, MakeFuture};

    trait ParseStr {
        fn put_str(&mut self, &str);
    }
    struct Port(Arc<Mutex<ParseStr>>);

    impl<'a, O, F: FnOnce(&str) -> O> ParseStr for FutureImpl<&'a str, O, F> {
        fn put_str(&mut self, val: &str) {
            let converted = self.convert()(val);
            self.put(converted);
        }
    }

    impl Port {
        fn put(self, val: &str) {
            self.0.lock().expect("port locked").put_str(val)
        }
    }

    #[test]
    fn test_int() {
        let mut lp = rotor_test::MockLoop::new(());
        let ref mut scope = lp.scope(1);
        let arc = new(scope, |x: &str| x.parse().unwrap());
        let future: Future<u64> = arc.clone().make_future();
        let value: String = String::from("10");
        Port(arc).put(&value[..]);
        assert_eq!(future.consume().unwrap(), 10);
    }

}
