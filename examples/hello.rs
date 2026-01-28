//! Simple example: opens a standalone window with labels
//!
//! Run with: cargo run --example hello

use masonry::core::NewWidget;
use masonry::properties::types::Length;
use masonry::widgets::{Flex, Label};
use masonry_baseview::{MasonryWindow, Size, WindowOpenOptions, WindowScalePolicy};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    println!("Opening window...");

    let options = WindowOpenOptions {
        title: "Masonry Baseview Example".into(),
        size: Size::new(400.0, 300.0),
        scale: WindowScalePolicy::SystemScaleFactor,
    };

    // This blocks until the window is closed
    MasonryWindow::open_blocking(options, || {
        Flex::column()
            .with_child(NewWidget::new(Label::new("Hello from masonry_baseview!")))
            .with_spacer(Length::px(20.0))
            .with_child(NewWidget::new(Label::new("This UI is rendered with Vello/wgpu")))
            .with_spacer(Length::px(20.0))
            .with_child(NewWidget::new(Label::new("Running inside baseview window")))
    });

    println!("Window closed.");
}
