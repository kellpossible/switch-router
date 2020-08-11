# switch-router [![github action badge](https://github.com/kellpossible/switch-router/workflows/Rust/badge.svg)](https://github.com/kellpossible/switch-router/actions?query=workflow%3ARust)

An alternate version of the [yew RouteService](https://github.com/yewstack/yew/blob/master/yew-router/src/service.rs), which aims to provide a more type safe API for setting routes, a `SwitchRoute` trait which can easily be implemented manually on a type. This library provides an abstraction trait for the routing service called `SwitchRouteService` that has two implementations:

+ `WebRouteService` - a fork of `yew-router`'s `RouteService`, with a simpler internal implementation by dropping support for `stdweb`.
+ `SimpleRouteService` - an implementation that does not rely on web APIs, useful for developing desktop applications with `web-view`.
