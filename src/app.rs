use crate::draw::{Align, Point, Rect};
use crate::widget::Widget;

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
    Connection, EventQueue, QueueHandle,
};

pub const WIDTH: u32 = 0;
pub const HEIGHT: u32 = 30;

pub struct App {
    pub connection: Connection,
    pub compositor: CompositorState,
    pub layer_shell: LayerShell,
    pub layer_surface: LayerSurface,
    pub pointer: Option<wl_pointer::WlPointer>,

    pub shm_state: Shm,
    pub pool: SlotPool,
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,

    pub should_exit: bool,
    pub width: u32,
    pub height: u32,
    pub redraw: bool,
    pub widgets: Vec<Box<dyn Widget>>,
    pub last_damage: Vec<Rect<u32>>,
}

impl App {
    pub fn new() -> (Self, EventQueue<Self>) {
        log::info!("Starting wayland client");
        let connection = Connection::connect_to_env().unwrap();

        let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();
        let qh = event_queue.handle();

        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");

        let surface = compositor.create_surface(&qh);
        let layer_surface =
            layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("bar-wlrs"), None);

        layer_surface.set_anchor(Anchor::BOTTOM.complement()); // anchor to all sides but the bottom
        layer_surface.set_size(WIDTH, HEIGHT);
        layer_surface.set_exclusive_zone(HEIGHT as i32);
        layer_surface.commit();

        let shm_state = Shm::bind(&globals, &qh).expect("wl_shm not available");

        let pool =
            SlotPool::new(4000 * HEIGHT as usize, &shm_state).expect("Failed to create pool");
        //                ^^^^ seems like a reasonable default, 4, 1000 size buffers

        //let text_box = crate::draw::TextBox::builder("test box".to_owned())
        //    .text("this is a test box".to_string())
        //    .desired_height(HEIGHT as f32)
        //    .build();

        let clock = crate::clock::Clock::new("clock".to_string(), HEIGHT as f32);

        let mut me = Self {
            connection,
            compositor,
            layer_shell,
            layer_surface,
            pointer: None,
            widgets: vec![Box::new(clock)],

            shm_state,
            pool,
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),

            should_exit: false,
            width: WIDTH,
            height: HEIGHT,
            redraw: true,
            last_damage: Vec::with_capacity(16),
        };

        event_queue
            .roundtrip(&mut me)
            .expect("failed to initialize");

        (me, event_queue)
    }
}

impl CompositorHandler for App {
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

impl OutputHandler for App {
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

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.should_exit = true;
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
            self.width = 1000; // reasonable default since the requested width would be 0 otherwise
            self.height = HEIGHT;
        } else {
            log::debug!(
                "new size requested ({}, {})",
                configure.new_size.0,
                configure.new_size.1
            );
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }

        let (width, height) = (self.width as f32, self.height as f32);
        let canvas_size = Point::new(width, height);
        let canvas = canvas_size.extend_to(0.0, 0.0);

        for w in self.widgets.iter_mut() {
            let wid_height = w.desired_height().clamp(0.0, height);
            let wid_width = w.desired_width(wid_height).clamp(0.0, width);

            let size = Point::new(wid_width, wid_height);

            let new_area = canvas.place_at(size, Align::Center, Align::Center);
            w.resize(new_area);
        }

        self.draw(qh);
    }
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl SeatHandler for App {
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

impl PointerHandler for App {
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
            if &event.surface != self.layer_surface.wl_surface() {
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

impl App {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        self.pool
            .resize((self.width * self.height * 4) as usize)
            .unwrap();
        let stride = self.width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                self.width as i32,
                self.height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .unwrap();

        let rect = Point::new(0, 0).extend_to(self.width, self.height);

        let surface = self.layer_surface.wl_surface();

        let mut ctx = crate::draw::DrawCtx {
            damage: &mut vec![],
            buffer: &buffer,
            canvas,
            rect,
            full_redraw: self.redraw,
        };

        //for dam in self.last_damage.iter() {
        //    dam.draw_outline(*crate::color::SURFACE, &mut ctx);
        //    dam.damage_outline(surface.clone());
        //}
        //debug_assert!(ctx.damage.is_empty());

        self.last_damage.clear();
        ctx.damage = &mut self.last_damage;

        if self.redraw {
            rect.draw(*crate::color::SURFACE, &mut ctx);
            //rect.draw_outline(*crate::color::PINE, &mut ctx);
        }

        for w in self.widgets.iter_mut() {
            if let Err(err) = w.draw(&mut ctx) {
                log::warn!("widget failed to draw: error={err}");
            }
        }

        if self.redraw {
            self.redraw = false;
            log::debug!("full redraw");

            // Damage the entire window
            surface.damage_buffer(0, 0, self.width as i32, self.height as i32);
            ctx.damage.clear();
        } else {
            let damage = ctx.damage.clone();
            for dam in damage {
                surface.damage_buffer(
                    dam.min.x as i32,
                    dam.min.y as i32,
                    dam.max.x as i32,
                    dam.max.y as i32,
                );

                //dam.draw_outline(*crate::color::LOVE, &mut ctx);
            }
        }

        surface.frame(qh, surface.clone()); // Request our next frame
        ctx.buffer.attach_to(surface).unwrap();

        self.layer_surface.commit();

        // hack to test all sizes above your own (until it hits some limit)
        //self.layer_surface.set_size(WIDTH, self.height + 1);
        //self.layer_surface
        //    .set_exclusive_zone(self.height as i32 + 1);
        //self.layer_surface.commit();
    }

    pub fn run_queue(&mut self, event_queue: &mut EventQueue<Self>) {
        loop {
            if let Err(err) = event_queue.blocking_dispatch(self) {
                log::warn!("event queue error: error={err}");
            }

            if self.should_exit {
                log::info!("exiting...");
                break;
            }
        }
    }
}

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);

delegate_seat!(App);
delegate_pointer!(App);

delegate_layer!(App);
delegate_registry!(App);

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
