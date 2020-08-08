use super::{notify_callbacks, SwitchRouteService};
use crate::{route::SwitchRoute, Listener};
use gloo_events::EventListener;
use std::{cell::RefCell, fmt::Debug, marker::PhantomData, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::{History, Location};

type ListenersRef<R> = Rc<RefCell<Vec<Listener<R>>>>;

#[derive(Debug)]
pub struct WebRouteService<R> {
    history: History,
    location: Location,
    listeners: ListenersRef<R>,
    event_listener: EventListener,
    switch_route_type: PhantomData<R>,
}

impl<R> PartialEq for WebRouteService<R>
where
    R: SwitchRoute + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_route() == other.get_route()
    }
}

impl<R> WebRouteService<R>
where
    R: SwitchRoute + 'static,
{
    pub fn new() -> Self {
        let window = web_sys::window().expect("browser does not have a window");

        let history = window
            .history()
            .expect("browser does not support the history API");

        let location = window.location();

        let listeners = Rc::new(RefCell::new(Vec::new()));
        let listeners_callbacks = listeners.clone();

        let event_listener = EventListener::new(&window, "popstate", move |_event| {
            let location = web_sys::window()
                .expect("browser does not have a window")
                .location();
            let route = Self::route_from_location(&location);

            let mut listeners_callbacks_ref = listeners_callbacks.borrow_mut();
            notify_callbacks(&mut *listeners_callbacks_ref, &route);
        });

        Self {
            history,
            location,
            listeners,
            event_listener,
            switch_route_type: PhantomData,
        }
    }

    fn route_from_location(location: &Location) -> R {
        let route = format!(
            "{pathname}{search}{hash}",
            pathname = location.pathname().unwrap(),
            search = location.search().unwrap(),
            hash = location.hash().unwrap()
        );

        R::switch(&route)
    }
}

impl<R> Default for WebRouteService<R>
where
    R: SwitchRoute + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R> SwitchRouteService for WebRouteService<R>
where
    R: SwitchRoute + 'static,
{
    type Route = R;
    fn set_route<RI: Into<Self::Route>>(&mut self, route: RI){
        let route = route.into();
        //TODO: replace null with actual state storage
        self.history
            .push_state_with_url(&JsValue::null(), "", Some(&route.path()))
            .unwrap();

        let mut listeners_ref = self.listeners.borrow_mut();
        // TODO: only notify callbacks if the route actually changed?
        notify_callbacks(&mut *listeners_ref, &route);
    }

    fn replace_route<RI: Into<Self::Route>>(&mut self, route: RI) -> Self::Route {
        let route = route.into();
        let return_route = self.get_route();
        //TODO: replace null with actual state storage
        self.history
            .replace_state_with_url(&JsValue::null(), "", Some(&route.path()))
            .unwrap();

        let mut listeners_ref = self.listeners.borrow_mut();
        notify_callbacks(&mut *listeners_ref, &route);
        return_route
    }

    fn get_route(&self) -> Self::Route {
        Self::route_from_location(&self.location)
    }

    fn register_callback<L: crate::listener::AsListener<R = Self::Route>>(&mut self, listener: L) {
        self.listeners.borrow_mut().push(listener.as_listener());
    }

    fn back(&mut self) -> Option<Self::Route> {
        unimplemented!()
    }
}
