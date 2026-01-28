//! Masonry Baseview Backend
//!
//! This crate provides a baseview backend for Masonry/Xilem, enabling
//! Xilem-based UIs to be used in audio plugin contexts (CLAP, VST3, etc).
//!
//! # Status
//!
//! **Experimental** - This is a work-in-progress integration.
//!
//! A feature request was opened with the Xilem team for official baseview support:
//! <https://github.com/linebender/xilem/issues/1626>
//!
//! # Architecture
//!
//! Unlike masonry_winit which uses winit's event loop, this crate uses
//! baseview's WindowHandler callback model. This is necessary because:
//!
//! 1. Audio plugin hosts own the main thread/event loop
//! 2. Baseview supports embedding into host-provided parent windows
//! 3. Plugins need to integrate with the host's window system
//!
//! # Usage
//!
//! ```ignore
//! use masonry_baseview::{MasonryWindow, WindowOpenOptions, Size, WindowScalePolicy};
//! use masonry::widgets::Label;
//!
//! // For CLAP plugins with parent window:
//! MasonryWindow::open_parented(
//!     parent_handle,
//!     WindowOpenOptions {
//!         title: "My Plugin".into(),
//!         size: Size::new(800.0, 600.0),
//!         scale: WindowScalePolicy::SystemScaleFactor,
//!     },
//!     || Label::new("Hello from masonry!"),
//! );
//! ```

mod event;
mod render;
mod window;

pub use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
pub use window::{MasonryWindow, MasonryWindowHandle};
