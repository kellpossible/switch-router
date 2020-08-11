use super::notify_callbacks;
use crate::{route::SwitchRoute, Listener, SwitchRouteService};
use std::marker::PhantomData;

struct History<R> {
    stack: Vec<R>,
    history_pointer: usize,
}

pub type RouteChanged = bool;

impl<R> History<R>
where
    R: SwitchRoute + Clone,
{
    pub fn new(initial_route: R) -> Self {
        Self {
            stack: vec![initial_route],
            history_pointer: 0,
        }
    }

    pub fn push(&mut self, route: R) -> RouteChanged {
        if self.get_current() != route {
            if self.history_pointer < self.stack.len() - 1 {
                self.stack = Vec::from(&self.stack[0..=self.history_pointer]);
            }
            self.stack.push(route);
            self.history_pointer += 1;
            true
        } else {
            false
        }
    }

    pub fn get_current(&self) -> R {
        self.stack
            .get(self.history_pointer)
            .expect("expected a stack item to match the history pointer")
            .clone()
    }

    pub fn replace_current(&mut self, route: R) -> R {
        let removed = self.stack.remove(self.history_pointer);
        self.stack.insert(self.history_pointer - 1, route);
        removed
    }

    pub fn back(&mut self) -> Option<R> {
        if self.history_pointer > 0 {
            self.history_pointer -= 1;
            Some(self.get_current())
        } else {
            None
        }
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
    R: SwitchRoute + Clone + 'static,
{
    type Route = R;

    fn set_route<RI: Into<Self::Route>>(&mut self, route: RI) {
        let route = route.into();
        self.history.push(route.clone());
        notify_callbacks(&mut self.listeners, &route);
    }

    fn replace_route<RI: Into<Self::Route>>(&mut self, route: RI) -> Self::Route {
        let route = route.into();
        let old_route = self.history.replace_current(route.clone());
        notify_callbacks(&mut self.listeners, &route);
        old_route
    }

    fn get_route(&self) -> Self::Route {
        self.history.get_current()
    }

    fn register_callback<L: crate::listener::AsListener<R = Self::Route>>(&mut self, listener: L) {
        self.listeners.push(listener.as_listener());
    }

    fn back(&mut self) -> Option<Self::Route> {
        self.history.back().map(|route| {
            notify_callbacks(&mut self.listeners, &route);
            route
        })
    }
}
