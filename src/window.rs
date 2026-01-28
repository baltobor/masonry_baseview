//! Masonry window implementation using baseview
//!
//! Provides the main window handler that integrates masonry's RenderRoot
//! with baseview's window system.

use crate::event::{EventTranslator, MasonryEvent};
use crate::render::RenderContext;
use baseview::{Event, EventStatus, Window, WindowHandler, WindowOpenOptions};
use masonry::app::{RenderRoot, RenderRootOptions, WindowSizePolicy};
use masonry::core::{NewWidget, Widget, WindowEvent as MasonryWindowEvent};
use masonry::theme::default_property_set;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;
use std::time::Instant;
use vello::peniko::Color;
use vello::Scene;

/// Handle to a masonry window running in baseview
pub struct MasonryWindowHandle {
    // Currently empty - baseview handles are fire-and-forget
    // In future could add communication channel
}

/// Builder for creating masonry windows with deferred widget creation
pub struct MasonryWindow;

impl MasonryWindow {
    /// Open a window parented to another window (for plugin UIs)
    ///
    /// This is the primary method for CLAP/VST plugin integration.
    /// The widget_builder closure will be called on the window thread to create the widget.
    pub fn open_parented<P, B, W>(
        parent: &P,
        options: WindowOpenOptions,
        widget_builder: B,
    ) -> MasonryWindowHandle
    where
        P: HasRawWindowHandle,
        B: FnOnce() -> W + Send + 'static,
        W: Widget + 'static,
    {
        let width = options.size.width;
        let height = options.size.height;

        // Wrap the builder in Option so we can take it once
        let builder_cell = std::sync::Mutex::new(Some(widget_builder));

        Window::open_parented(parent, options, move |_| {
            // Take the builder out of the mutex - this runs on the window thread
            let builder = builder_cell.lock().unwrap().take().unwrap();
            MasonryHandler::new(builder, width, height)
        });

        MasonryWindowHandle {}
    }

    /// Open a standalone window (for testing)
    ///
    /// Note: This blocks the current thread until the window is closed.
    /// Due to RenderRoot's internal structure, this must be called from
    /// the main thread on macOS.
    pub fn open_blocking<B, W>(options: WindowOpenOptions, widget_builder: B)
    where
        B: FnOnce() -> W + Send + 'static,
        W: Widget + 'static,
    {
        let width = options.size.width;
        let height = options.size.height;

        let builder_cell = std::sync::Mutex::new(Some(widget_builder));

        Window::open_blocking(options, move |_| {
            let builder = builder_cell.lock().unwrap().take().unwrap();
            MasonryHandler::new(builder, width, height)
        });
    }
}

/// Internal window handler that bridges baseview to masonry
///
/// This uses a two-phase initialization:
/// 1. The handler is created with just the widget builder (Send-safe)
/// 2. On first frame, the widget and RenderRoot are created (non-Send, but on window thread)
struct MasonryHandler<W: Widget + 'static> {
    /// Widget builder - consumed on first frame to create widget
    widget_builder: Option<Box<dyn FnOnce() -> W + Send>>,
    /// The masonry render root (created lazily)
    render_root: Option<RenderRoot>,
    /// GPU rendering context
    render_ctx: Option<RenderContext>,
    /// Event translator
    event_translator: EventTranslator,
    /// Current scene
    scene: Scene,
    /// Last frame time
    last_frame: Instant,
    /// Background color
    base_color: Color,
    /// Window dimensions
    width: f64,
    height: f64,
}

impl<W: Widget + 'static> MasonryHandler<W> {
    fn new<B>(widget_builder: B, width: f64, height: f64) -> Self
    where
        B: FnOnce() -> W + Send + 'static,
    {
        Self {
            widget_builder: Some(Box::new(widget_builder)),
            render_root: None,
            render_ctx: None,
            event_translator: EventTranslator::new(1.0),
            scene: Scene::new(),
            last_frame: Instant::now(),
            base_color: Color::from_rgba8(30, 30, 35, 255), // Dark background
            width,
            height,
        }
    }

    fn ensure_initialized(&mut self, window: &mut Window) {
        // Initialize GPU context
        if self.render_ctx.is_none() {
            match unsafe { RenderContext::new(window, self.width as u32, self.height as u32) } {
                Ok(ctx) => {
                    self.render_ctx = Some(ctx);
                    tracing::info!("GPU context initialized");
                }
                Err(e) => {
                    tracing::error!("Failed to create GPU context: {}", e);
                    return;
                }
            }
        }

        // Initialize RenderRoot with the widget
        if self.render_root.is_none() {
            if let Some(builder) = self.widget_builder.take() {
                let widget = builder();
                let new_widget = NewWidget::new(widget);

                let options = RenderRootOptions {
                    default_properties: Arc::new(default_property_set()),
                    use_system_fonts: true,
                    size_policy: WindowSizePolicy::User,
                    size: masonry::dpi::PhysicalSize::new(self.width as u32, self.height as u32),
                    scale_factor: 1.0,
                    test_font: None,
                };

                // Create render root with signal sink
                let render_root = RenderRoot::new(new_widget, |_signal| {}, options);
                self.render_root = Some(render_root);

                tracing::info!("Widget tree initialized");
            }
        }
    }

    fn handle_masonry_event(&mut self, event: MasonryEvent) {
        let Some(render_root) = &mut self.render_root else {
            return;
        };

        match event {
            MasonryEvent::Pointer(ptr_event) => {
                let _ = render_root.handle_pointer_event(ptr_event);
            }
            MasonryEvent::Keyboard(_kb_event) => {
                // TODO: Implement keyboard event handling
                // Would need to convert keyboard_types to masonry's TextEvent
            }
            MasonryEvent::Resize { width, height, scale } => {
                self.width = width / scale;
                self.height = height / scale;
                self.event_translator.set_scale_factor(scale);

                if let Some(ctx) = &mut self.render_ctx {
                    ctx.resize(width as u32, height as u32);
                }

                // Send resize and rescale events
                let _ = render_root.handle_window_event(MasonryWindowEvent::Resize(
                    masonry::dpi::PhysicalSize::new(width as u32, height as u32),
                ));
                let _ = render_root.handle_window_event(MasonryWindowEvent::Rescale(scale));
            }
            MasonryEvent::Focus(_focused) => {
                // Masonry doesn't have focus events in WindowEvent
                // Focus tracking is handled internally by pointer/keyboard events
            }
            MasonryEvent::Close => {
                // Window closing - cleanup handled by drop
            }
        }
    }

    fn render_frame(&mut self) {
        // Skip rendering entirely until both render_root and render_ctx are initialized
        // This prevents showing garbage/triangle on the first frame
        if self.render_root.is_none() || self.render_ctx.is_none() {
            return;
        }
        let render_root = self.render_root.as_mut().unwrap();
        let render_ctx = self.render_ctx.as_mut().unwrap();

        // Calculate animation delta
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame);
        self.last_frame = now;

        // Send animation frame event
        let _ = render_root.handle_window_event(MasonryWindowEvent::AnimFrame(dt));

        // Get the rendered scene from masonry
        let (scene, _accessibility) = render_root.redraw();
        self.scene = scene;

        // Render to surface
        if let Err(e) = render_ctx.render(&self.scene, self.base_color) {
            tracing::error!("Render error: {}", e);
        }
    }
}

impl<W: Widget + 'static> WindowHandler for MasonryHandler<W> {
    fn on_frame(&mut self, window: &mut Window) {
        self.ensure_initialized(window);
        self.render_frame();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        if let Some(masonry_event) = self.event_translator.translate(&event) {
            self.handle_masonry_event(masonry_event);
            EventStatus::Captured
        } else {
            EventStatus::Ignored
        }
    }
}
