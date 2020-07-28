# yew-switch-router [![github action badge](https://github.com/kellpossible/yew-switch-router/workflows/Rust/badge.svg)](https://github.com/kellpossible/yew-switch-router/actions?query=workflow%3ARust)

An alternate version of the [yew RouteService](https://github.com/yewstack/yew/blob/master/yew-router/src/service.rs), which aims to provide a more type safe API for setting routes, a `SwitchRoute` trait which can easily be implemented manually on a type, and a much simpler internal implementation by dropping support for `stdweb`.
