use std::sync::{Arc, Mutex};

use rotor::{Notifier, GenericScope};


pub struct Future<T>(Arc<Mutex<Option<T>>>);

pub struct Port<A: ?Sized, B: 'static>(Arc<Mutex<Option<B>>>, Notifier, fn(&A) -> B);

pub fn pair<S: GenericScope, A: ?Sized, B: 'static>(scope: &mut S, fun: fn(&A) -> B)
    -> (Future<B>, Port<A, B>)
{
    let arc = Arc::new(Mutex::new(None));
    (Future(arc.clone()), Port(arc, scope.notifier(), fun))
}

impl<A: ?Sized, B: 'static> Port<A, B> {
    /// Put the value into corresponding future
    ///
    /// Returns true if value is put (otherwise future is probably
    /// already destoryed)
    pub fn put(mut self, value: &A) -> bool {
        let ref fun = self.2;
        let real_value = fun(value);
        let ok = self.0.lock().as_mut().map(|x| {
            **x = Some(real_value)
        }).is_ok();
        // TODO(tailhook) this should be strong_count() but that is unstable
        self.1.wakeup().is_ok() && ok && Arc::get_mut(&mut self.0).is_none()
    }
}

impl<T: Sized> Future<T> {
    pub fn is_done(&self) -> bool {
        self.0.lock().expect("future can be locked").is_some()
    }
    pub fn consume(self) -> Result<T, Self> {
        match self.0.lock().expect("future can be locked").take() {
            Some(x) => return Ok(x),
            None => {}
        }
        Err(self)
    }
}

#[cfg(test)]
mod test {
    extern crate rotor_test;

    use std::str::FromStr;
    use super::{pair};

    #[test]
    fn test_int() {
        let mut lp = rotor_test::MockLoop::new(());
        let ref mut scope = lp.scope(1);
        let (future, port) = pair(scope, u32::from_str);
        let value: String = String::from("10");
        port.put(&value[..]);
        assert_eq!(future.consume().unwrap_or(Ok(9999)), Ok(10));
    }

}
