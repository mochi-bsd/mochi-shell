/// RSX-like macro system for declarative UI
/// 
/// Provides a more ergonomic, React/SwiftUI-like syntax for building UIs

/// Macro for creating UI trees with JSX-like syntax
#[macro_export]
macro_rules! view {
    // Container with children
    (Container {
        $($prop:ident: $val:expr,)*
        children: [$($child:expr),* $(,)?]
    }) => {{
        let mut container = $crate::container(0, 0, 0, 0);
        $(
            container = view!(@prop container, $prop, $val);
        )*
        $(
            container = container.child($child);
        )*
        container
    }};

    // Container without children
    (Container {
        $($prop:ident: $val:expr),* $(,)?
    }) => {{
        let mut container = $crate::container(0, 0, 0, 0);
        $(
            container = view!(@prop container, $prop, $val);
        )*
        container
    }};

    // Card with children
    (Card {
        $($prop:ident: $val:expr,)*
        children: [$($child:expr),* $(,)?]
    }) => {{
        let mut card = $crate::card(0, 0, 0, 0);
        $(
            card = view!(@prop card, $prop, $val);
        )*
        $(
            card = card.child($child);
        )*
        card
    }};

    // Card without children
    (Card {
        $($prop:ident: $val:expr),* $(,)?
    }) => {{
        let mut card = $crate::card(0, 0, 0, 0);
        $(
            card = view!(@prop card, $prop, $val);
        )*
        card
    }};

    // Text element
    (Text {
        $($prop:ident: $val:expr),* $(,)?
    }) => {{
        let mut text = $crate::text("", 0, 0);
        $(
            text = view!(@prop text, $prop, $val);
        )*
        text
    }};

    // VStack with children
    (VStack {
        $($prop:ident: $val:expr,)*
        children: [$($child:expr),* $(,)?]
    }) => {{
        let mut vstack = $crate::vstack(0, 0);
        $(
            vstack = view!(@prop vstack, $prop, $val);
        )*
        $(
            vstack = vstack.child($child);
        )*
        vstack
    }};

    // VStack without children
    (VStack {
        $($prop:ident: $val:expr),* $(,)?
    }) => {{
        let mut vstack = $crate::vstack(0, 0);
        $(
            vstack = view!(@prop vstack, $prop, $val);
        )*
        vstack
    }};

    // Titlebar
    (Titlebar {
        $($prop:ident: $val:expr),* $(,)?
    }) => {{
        let mut titlebar = $crate::titlebar(0, "");
        $(
            titlebar = view!(@prop titlebar, $prop, $val);
        )*
        titlebar
    }};

    // Property setters
    (@prop $elem:expr, x, $val:expr) => { $elem };
    (@prop $elem:expr, y, $val:expr) => { $elem };
    (@prop $elem:expr, width, $val:expr) => { $elem };
    (@prop $elem:expr, height, $val:expr) => { $elem };
    (@prop $elem:expr, background, $val:expr) => { $elem.background($val) };
    (@prop $elem:expr, color, $val:expr) => { $elem.color($val) };
    (@prop $elem:expr, rounded, $val:expr) => { $elem.rounded($val) };
    (@prop $elem:expr, blur, $val:expr) => { $elem.blur($val) };
    (@prop $elem:expr, glow, $val:expr) => { $elem.glow($val) };
    (@prop $elem:expr, brightness, $val:expr) => { $elem.brightness($val) };
    (@prop $elem:expr, gradient, $val:expr) => { $elem.gradient($val.0, $val.1) };
    (@prop $elem:expr, shadow, $val:expr) => { $elem.shadow($val) };
    (@prop $elem:expr, shadow_blur, $val:expr) => { $elem.shadow_blur($val) };
    (@prop $elem:expr, text, $val:expr) => { $elem };
    (@prop $elem:expr, size, $val:expr) => { $elem.size($val) };
    (@prop $elem:expr, font, $val:expr) => { $elem.font($val) };
    (@prop $elem:expr, title, $val:expr) => { $elem };
    (@prop $elem:expr, spacing, $val:expr) => { $elem.spacing($val) };
}

/// Builder pattern helpers for more ergonomic positioning
pub trait Positioned {
    fn at(self, x: i32, y: i32) -> Self;
    fn size(self, width: i32, height: i32) -> Self;
    fn frame(self, x: i32, y: i32, width: i32, height: i32) -> Self;
}

/// Layout helpers
pub trait Layout {
    fn padding(self, padding: i32) -> Self;
    fn margin(self, margin: i32) -> Self;
}
