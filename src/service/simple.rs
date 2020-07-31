use std::{marker::PhantomData, cell::RefCell};
use crate::{SwitchRouteService, route::SwitchRoute, Listener};

struct History<R> {
    stack: Vec<R>,
    history_pointer: usize,
}

impl<R: Clone> History<R> {
    pub fn new(initial_route: R) -> Self {
        Self {
            stack: vec![initial_route],
            history_pointer: 1
        }
    }

    pub fn push(&mut self, route: R) {
        self.stack.push(route);
        self.history_pointer += 1;
    }

    pub fn get_current(&self) -> R {
        self.stack.get(self.history_pointer).expect("expected a stack item to match the history pointer").clone()
    }

    pub fn replace_current(&mut self, route: R) -> R {
        let removed = self.stack.remove(self.history_pointer);
        self.stack.insert(self.history_pointer-1, route);
        removed
    }
}

pub struct SimpleRouteService<R> {
    history: History<R>,
    listeners: Vec<Listener<R>>,
    switch_route_type: PhantomData<R>,
}

impl<R> SimpleRouteService<R>
where
    R: SwitchRoute + 'static,
{
    pub fn new(initial_route: R) -> Self {
        let history = History::new(initial_route);
        let listeners = Vec::new();

        Self {
            history,
            listeners,
            switch_route_type: PhantomData,
        }
    }
}

impl<R> SwitchRouteService for SimpleRouteService<R> 
where
    R: SwitchRoute + Clone + 'static {
    type Route = R;

    fn set_route<RI: Into<Self::Route>>(&mut self, route: RI) {
        let route = route.into();
        self.history.push(route);
    }
    
    fn replace_route<RI: Into<Self::Route>>(&mut self, route: RI) -> Self::Route {
        let route = route.into();
        self.history.replace_current(route)
    }

    fn get_route(&self) -> Self::Route {
        self.history.get_current()
    }

    fn register_callback<L: crate::listener::AsListener<R = Self::Route>>(&mut self, listener: L) {
        self.listeners.push(listener.as_listener());
    }
}
