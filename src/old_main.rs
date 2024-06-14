pub mod buffer;
pub mod color;
pub mod draw;
pub mod utils;
pub mod widget;

pub mod clock;

use draw::*;
use widget::Widget;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

const HEIGHT: usize = 32;
const WIDTH: usize = 0;

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let surface = compositor.create_surface(&qh);

    let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("bar-wlrs"), None);

    let (width, height) = (WIDTH as u32, HEIGHT as u32);

    // get every layer besides for the bottom
    const ANCHOR: Anchor = Anchor::BOTTOM.complement();
    layer.set_anchor(ANCHOR);
    layer.set_size(width, height);
    layer.set_exclusive_zone(height as i32);

    layer.commit();

    let buffer = buffer::Buffer::new(&shm, width, height);

    let widgets: Vec<Box<dyn Widget>> = vec![Box::new(clock::Clock::new())];

    let mut bar_layer = BarLayer {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,

        buffer,

        exit: false,
        first_configure: true,
        first_draw: true,
        layer,
        pointer: None,

        widgets,
    };

    // We don't draw immediately, the configure will notify us when to first draw.
    loop {
        event_queue.blocking_dispatch(&mut bar_layer).unwrap();

        if bar_layer.exit {
            println!("exiting example");
            break;
        }
    }
}

struct BarLayer {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    buffer: buffer::Buffer,

    exit: bool,
    first_configure: bool,
    first_draw: bool,
    layer: LayerSurface,
    pointer: Option<wl_pointer::WlPointer>,

    widgets: Vec<Box<dyn Widget>>,
}

impl CompositorHandler for BarLayer {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }
}

impl OutputHandler for BarLayer {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for BarLayer {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
            self.buffer.width = WIDTH as u32;
            self.buffer.height = HEIGHT as u32;
        } else {
            log::debug!(
                "new size requested ({}, {})",
                configure.new_size.0,
                configure.new_size.1
            );
            self.buffer.width = configure.new_size.0;
            self.buffer.height = configure.new_size.1;
        }

        let canvas_size: Point = (self.buffer.width, self.buffer.height).into();
        let canvas = canvas_size.extend_to(0, 0);

        for w in self.widgets.iter_mut() {
            w.place(canvas, Align::Center, Align::Center);
        }

        // Initiate the first draw.
        self.draw(qh);
    }
}

impl SeatHandler for BarLayer {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            log::debug!("Set pointer capability");
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_some() {
            log::debug!("Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for BarLayer {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            // Ignore events for other surfaces
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            match event.kind {
                Enter { .. } => {
                    log::trace!("Pointer entered @{:?}", event.position);
                }
                Leave { .. } => {
                    log::trace!("Pointer left");
                }
                Motion { .. } => {}
                Press { button, .. } => {
                    log::trace!("Press {:x} @ {:?}", button, event.position);
                }
                Release { button, .. } => {
                    log::trace!("Release {:x} @ {:?}", button, event.position);
                }
                Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    log::trace!("Scroll H:{horizontal:?}, V:{vertical:?}");
                }
            }
        }
    }
}

impl ShmHandler for BarLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl BarLayer {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        if self.first_draw {
            self.buffer.fill(*color::SURFACE);

            return;
        }

        let canvas_size: Point = (self.buffer.width, self.buffer.height).into();

        for wid in self.widgets.iter_mut() {
            if let Err(err) = wid.draw(
                self.buffer
                    .buffer
                    .canvas(&mut self.buffer.pool)
                    .expect("no buffer?"),
                canvas_size,
            ) {
                log::error!("widget failed to draw. error={err}");
            }
        }

        //fn new(canvas_size: Point<u32>, pos: Point<u32>, size: Point<u32>) -> Self;

        // Damage the entire window
        self.layer.wl_surface().damage_buffer(
            0,
            0,
            self.buffer.width as i32,
            self.buffer.height as i32,
        );

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        self.buffer
            .buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }
}

delegate_compositor!(BarLayer);
delegate_output!(BarLayer);
delegate_shm!(BarLayer);

delegate_seat!(BarLayer);
delegate_pointer!(BarLayer);

delegate_layer!(BarLayer);

delegate_registry!(BarLayer);

impl ProvidesRegistryState for BarLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
