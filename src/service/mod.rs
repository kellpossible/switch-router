mod simple;
pub use simple::SimpleRouteService;

#[cfg(feature = "web")]
mod web;

#[cfg(feature = "web")]
pub use web::WebRouteService;

use crate::{listener::{AsListener, Listener}, route::SwitchRoute};
use std::cell::RefCell;

pub trait SwitchRouteService {
    type Route: SwitchRoute + 'static;

    fn set_route<RI: Into<Self::Route>>(&mut self, route: RI);
    fn replace_route<RI: Into<Self::Route>>(&mut self, route: RI) -> Self::Route;
    fn get_route(&self) -> Self::Route;
    fn register_callback<L: AsListener<R = Self::Route>>(&mut self, listener: L);
}

fn notify_callbacks<R: Clone>(listeners: &mut Vec<Listener<R>>, route: R) {
    let mut listeners_to_remove: Vec<usize> = Vec::new();
    for (i, listener) in listeners.iter().enumerate() {
        match &listener.callback() {
            Some(callback) => {
                callback.emit(route.clone());
            }
            None => {
                listeners_to_remove.push(i);
            }
        }
    }

    for i in listeners_to_remove {
        listeners.remove(i);
    }
}