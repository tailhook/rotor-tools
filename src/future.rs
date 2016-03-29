use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

use rotor::{Notifier, GenericScope};

trait ReadFuture<T> {
    fn ready(&self) -> bool;
    fn take(&mut self) -> Option<T>;
}

pub struct Future<T>(Arc<Mutex<ReadFuture<T>>>);

pub struct FutureImpl<I, O, F: FnOnce(I) -> O>{
    output: Option<O>,
    convert: Option<F>,
    notifier: Notifier,
    phantom: PhantomData<*const I>,
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

pub fn new<I, O: 'static, F:FnOnce(I) -> O + 'static, S: GenericScope>(scope: &mut S, fun: F)
    -> Arc<Mutex<FutureImpl<I, O, F>>>
{
    Arc::new(Mutex::new(FutureImpl::new(fun, scope.notifier())))
}

#[cfg(test)]
mod test {
    extern crate rotor_test;
    use std::sync::{Arc, Mutex};

    use std::str::FromStr;
    use super::{new, FutureImpl, Future};

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
        let future: Future<u64> = Future(arc.clone());
        let value: String = String::from("10");
        Port(arc).put(&value[..]);
        assert_eq!(future.consume().unwrap_or(9999), 10);
    }

}
