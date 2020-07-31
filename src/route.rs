pub trait SwitchRoute: Clone + PartialEq {
    fn is_invalid(&self) -> bool;
    fn path(&self) -> String;
    fn switch(route: &str) -> Self;
}