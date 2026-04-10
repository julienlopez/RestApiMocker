//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//!
//! The [`Home`] component will be rendered when the current route is [`Route::Home`].
//!
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod home;
pub use home::Home;

pub mod mock_config_modal;

mod mocks;
pub use mocks::Mocks;

mod navbar;
pub use navbar::Navbar;
