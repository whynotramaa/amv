pub mod audio;
pub mod auth;
pub mod context;
pub mod credentials;
pub mod dispatch;
pub mod documents;
#[cfg(feature = "desktop")]
mod file_picker;
pub mod inference;
#[cfg(feature = "desktop")]
pub mod meeting;
pub mod providers;
pub mod speech;
pub mod store;
pub mod window;

#[cfg(feature = "desktop")]
mod desktop;

#[cfg(feature = "desktop")]
pub fn run() {
    desktop::run();
}
