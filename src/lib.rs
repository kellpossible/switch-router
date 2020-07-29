use gloo_events::EventListener;
use std::{
    cell::RefCell,
    fmt::Debug,
    marker::PhantomData,
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use web_sys::{History, Location};

pub trait SwitchRoute: Clone + PartialEq {
    fn is_invalid(&self) -> bool;
    fn path(&self) -> String;
    fn switch(route: &str) -> Self;
}

#[derive(Clone)]
pub struct Callback<SR>(Rc<dyn Fn(SR)>);

impl<SR> Callback<SR> {
    pub fn new<F: Fn(SR) + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }
    pub fn emit(&self, args: SR) {
        self.0(args)
    }
}

impl<SR> Debug for Callback<SR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Callback({:p})", self.0)
    }
}

#[derive(Clone)]
pub struct Listener<SR>(Weak<dyn Fn(SR)>);

impl<SR> Listener<SR> {
    pub fn callback(&self) -> Option<Callback<SR>> {
        Weak::upgrade(&self.0).map(|rc| Callback(rc))
    }
}

impl<SR> Debug for Listener<SR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Weak::upgrade(&self.0) {
            Some(rc) => write!(f, "Listener({:p})", rc),
            None => write!(f, "Listener(None)"),
        }
    }
}

pub trait AsListener {
    type SR;
    fn as_listener(&self) -> Listener<Self::SR>;
}

impl<SR> AsListener for &Callback<SR> {
    type SR = SR;
    fn as_listener(&self) -> Listener<Self::SR> {
        Listener(Rc::downgrade(&self.0))
    }
}

impl<SR, F> From<F> for Callback<SR>
where
    F: Fn(SR) + 'static,
{
    fn from(f: F) -> Self {
        Callback(Rc::new(f))
    }
}

type ListenerVec<SR> = Rc<RefCell<Vec<Listener<SR>>>>;

#[derive(Debug)]
pub struct SwitchRouteService<SR> {
    history: History,
    location: Location,
    listeners: ListenerVec<SR>,
    event_listener: EventListener,
    switch_route_type: PhantomData<SR>,
}

impl<SR> PartialEq for SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_route() == other.get_route()
    }
}

impl<SR> SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    pub fn new() -> Self {
        let window = web_sys::window().expect("browser does not have a window");

        let history = window
            .history()
            .expect("browser does not support the history API");

        let location = window.location();

        let callbacks = Rc::new(RefCell::new(Vec::new()));
        let listener_callbacks = callbacks.clone();

        let event_listener = EventListener::new(&window, "popstate", move |_event| {
            let location = web_sys::window()
                .expect("browser does not have a window")
                .location();
            let route = Self::route_from_location(&location);
            Self::notify_callbacks(&listener_callbacks, route);
        });
        Self {
            history,
            location,
            listeners: callbacks,
            event_listener,
            switch_route_type: PhantomData::default(),
        }
    }

    pub fn set_route<SRI: Into<SR>>(&mut self, switch_route: SRI) {
        let route = switch_route.into();
        //TODO: replace null with actual state storage
        self.history
            .push_state_with_url(&JsValue::null(), "", Some(&route.path()))
            .unwrap();
        Self::notify_callbacks(&self.listeners, route);
    }

    pub fn replace_route<SRI: Into<SR>>(&mut self, switch_route: SRI) -> SR {
        let route = switch_route.into();
        let return_route = self.get_route();
        //TODO: replace null with actual state storage
        self.history
            .replace_state_with_url(&JsValue::null(), "", Some(&route.path()))
            .unwrap();
        Self::notify_callbacks(&self.listeners, route);
        return_route
    }

    fn route_from_location(location: &Location) -> SR {
        let route = format!(
            "{pathname}{search}{hash}",
            pathname = location.pathname().unwrap(),
            search = location.search().unwrap(),
            hash = location.hash().unwrap()
        );

        SR::switch(&route)
    }

    pub fn get_route(&self) -> SR {
        Self::route_from_location(&self.location)
    }

    fn notify_callbacks(listeners: &ListenerVec<SR>, switch_route: SR) {
        let mut listeners_to_remove: Vec<usize> = Vec::new();
        for (i, listener) in RefCell::borrow(&*listeners).iter().enumerate() {
            match &listener.callback() {
                Some(callback) => {
                    callback.emit(switch_route.clone());
                }
                None => {
                    listeners_to_remove.push(i);
                }
            }
        }

        let mut listeners_mut = RefCell::borrow_mut(&*listeners);
        for i in listeners_to_remove {
            listeners_mut.remove(i);
        }
    }

    pub fn register_callback<L: AsListener<SR = SR>>(&mut self, listener: L) {
        self.listeners.borrow_mut().push(listener.as_listener());
    }
}

impl<SR> Default for SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "yew")]
impl<SR> From<yew::Callback<SR>> for Callback<SR>
where
    SR: 'static,
{
    fn from(yew_callback: yew::Callback<SR>) -> Self {
        Self::from(move |route| yew_callback.emit(route))
    }
}
