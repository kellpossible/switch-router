use std::{
    fmt::Debug,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct Callback<R>(Rc<dyn Fn(R)>);

impl<R> Callback<R> {
    pub fn new<F: Fn(R) + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }
    pub fn emit(&self, args: R) {
        self.0(args)
    }
}

impl<R> Debug for Callback<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Callback({:p})", self.0)
    }
}

#[derive(Clone)]
pub struct Listener<R>(Weak<dyn Fn(R)>);

impl<R> Listener<R> {
    pub fn callback(&self) -> Option<Callback<R>> {
        Weak::upgrade(&self.0).map(|rc| Callback(rc))
    }
}

impl<R> Debug for Listener<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Weak::upgrade(&self.0) {
            Some(rc) => write!(f, "Listener({:p})", rc),
            None => write!(f, "Listener(None)"),
        }
    }
}

pub trait AsListener {
    type R;
    fn as_listener(&self) -> Listener<Self::R>;
}

impl<R> AsListener for &Callback<R> {
    type R = R;
    fn as_listener(&self) -> Listener<Self::R> {
        Listener(Rc::downgrade(&self.0))
    }
}

impl<R, F> From<F> for Callback<R>
where
    F: Fn(R) + 'static,
{
    fn from(f: F) -> Self {
        Callback(Rc::new(f))
    }
}

#[cfg(feature = "yew")]
impl<R> From<yew::Callback<R>> for Callback<R>
where
    R: 'static,
{
    fn from(yew_callback: yew::Callback<R>) -> Self {
        Self::from(move |route| yew_callback.emit(route))
    }
}
