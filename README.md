# masonry_baseview

An experimental baseview backend for the [Masonry](https://github.com/linebender/xilem) framework which is the foundation of xilem

This project is an experimental attempt to bridge between /Xilem and [Baseview](https://github.com/RustAudio/baseview) for creating plugin UIs.

**Status:** Experimental. A feature request with the Xilem team for official baseview support was issued: 
https://github.com/linebender/xilem/issues/1626

## The Problem

When you create a CLAP/VST plugin, your code runs inside a **host application** (like Bitwig, Cubase, Reaper, Ableton, Presonus Studio one, Apple Logic). The host owns the main thread and the event loop. Your plugin can't create its own window the normal way (like a desktop app would) because:

1. The host provides a **parent window handle** where your UI must be embedded
2. You can't run your own event loop - you'd block the host
3. On macOS, GUI frameworks typically require main thread ownership

## What Baseview Does

Baseview solves this by:

1. **Accepting a parent window handle** from the host and creating a child window inside it
2. **Providing callbacks** (`on_frame`, `on_event`) instead of running its own event loop - the host drives the timing
3. **Working cross-platform** (macOS, Windows, Linux)

```
┌─────────────────────────────────────────┐
│  DAW Host (Bitwig, Reaper, etc.)        │
│  ┌───────────────────────────────────┐  │
│  │  Plugin Window (host provides)    │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  Baseview Child Window      │  │  │
│  │  │  ┌───────────────────────┐  │  │  │
│  │  │  │  Your Plugin UI       │  │  │  │
│  │  │  │  (Masonry/Xilem)      │  │  │  │
│  │  │  └───────────────────────┘  │  │  │
│  │  └─────────────────────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## masonry_baseview's Role

**masonry_baseview** could bridge two things:

- **Masonry** (Xilem's widget toolkit) - knows how to layout and render widgets
- **Baseview** - knows how to create plugin windows and receive events

Without this bridge, Xilem/Masonry can only run in standalone applications using winit, which doesn't work inside plugins.

## In Your Plugin

The flow is:

1. Host calls your plugin's `gui_create()` with a parent window handle
2. You call `MasonryWindow::open_parented(parent_handle, ...)`
3. Baseview creates a child window embedded in the host's window
4. Masonry renders your UI into that window via Vello/wgpu
5. Host drives the frame updates, baseview forwards events to masonry

## Usage

```rust
use masonry::widgets::Label;
use masonry_baseview::{MasonryWindow, Size, WindowOpenOptions, WindowScalePolicy};

// For CLAP/VST plugins with parent window:
MasonryWindow::open_parented(
    parent_handle,
    WindowOpenOptions {
        title: "My Plugin".into(),
        size: Size::new(800.0, 600.0),
        scale: WindowScalePolicy::SystemScaleFactor,
    },
    || Label::new("Hello from masonry!"),
);

// For standalone testing:
MasonryWindow::open_blocking(options, || my_widget());
```

## Architecture

- **Event translation** - Converts baseview mouse/keyboard/window events to masonry pointer events
- **GPU rendering** - Vello rendering pipeline with intermediate texture blitting (required because Vello uses compute shaders that can't directly target surface textures)
- **Deferred initialization** - Widget builder is Send-safe, RenderRoot is created on window thread to handle threading constraints

## Contributing

Contributions are welcome! I propose this project as an official Xilem backend variant. I do not intend to become an official maintainer. This project is for research and documentation purposes only. I accept no liability for damages and cannot guarantee that this project will remain up to date.

See the [Xilem feature request](https://github.com/linebender/xilem/issues/1626) for discussion.

## License

Apache-2.0 (same as Xilem/Masonry)

## References

The Rust audio ecosystem already has solutions:

- [baseview](https://github.com/RustAudio/baseview) - A windowing library specifically for plugin UIs, supporting parent window embedding
- [egui-baseview](https://github.com/billydm/egui-baseview) and [iced-baseview](https://github.com/billydm/iced_baseview) exist, proving the pattern works.
- [CLACK](https://github.com/prokopyl/clack) is a framework, which offers rust code to create CLAP Audio plugins.
- [Writing a CLAP synthesizer in rust](https://kwarf.com/2024/07/writing-a-clap-synthesizer-in-rust-part-1/)
- [Octasine](https://www.octasine.com/) Rust based FM-Synth developed in iced

It would be great to see xilem as the number one choice for complex UI, even in cross platform plugins of all kind (not only audio).

## Quickstart

I added a justfile to the project. `just` is kond of a makefile in the rust universe.
You can install it by using cargo:
```shell
cargo install just
```

Run "Hello World" example
```shell
just hello
```

Other just targets:

|Command|Task|
|-------|----|
|just hello|run "Hello World"|
|just build|Build all examples|
|just check|Check compilation|  
|just test |Run tests|
|just clean|Clean build artifacts|
|just fmt|Format code|
|just lint|Run clippy lints|

## Screenshots

### MacOS

<img width="512" height="444" alt="masonry_baseview_macOS" src="https://github.com/user-attachments/assets/166ac60f-6500-4f70-a5a4-287b49768a6c" />
