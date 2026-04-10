use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div { id: "navbar",
            Link { to: Route::Home {}, "History" }
            Link { to: Route::Mocks {}, "Mocks" }
            div { id: "navbar-filler", "RestMocker" }
            ConfigBox {}
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}

#[component]
fn ConfigBox() -> Element {
    let config = use_resource(crate::requests::ui_requests::get_config);
    rsx! {
        div { id: "navbar-configbox",

            match &*config.read_unchecked() {
                Some(Ok(config)) => rsx! {
                    div { "Public Port: {config.public_port}" }
                    div { "Private Port: {config.private_port}" }
                },
                Some(Err(e)) => rsx! {
                    p { "Loading config failed, {e}" }
                },
                None => rsx! {
                    p { "Loading..." }
                },
            }
        }
    }
}
