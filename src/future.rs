use std::sync::{Arc, Mutex};

use rotor::{Notifier, GenericScope};


pub struct Future<T: Sized>(Arc<Mutex<Option<T>>>);

pub struct Port<T: Sized>(Arc<Mutex<Option<T>>>, Notifier);


pub fn pair<S: GenericScope, T>(scope: &mut S) -> (Future<T>, Port<T>) {
    let arc = Arc::new(Mutex::new(None));
    (Future(arc.clone()), Port(arc, scope.notifier()))
}

impl<T: Sized> Port<T> {
    /// Put the value into corresponding future
    ///
    /// Returns true if value is put (otherwise future is probably
    /// already destoryed)
    pub fn put(mut self, value: T) -> bool {
        let ok = self.0.lock().as_mut().map(|x| {
            **x = Some(value)
        }).is_ok();
        // TODO(tailhook) this should be strong_count() but that is unstable
        ok && Arc::get_mut(&mut self.0).is_none()
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

