use super::draw::{color, prelude::*};
use super::widget::{ClickType, Widget};
use crate::log::*;

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

pub struct App {
    //connection: Connection,
    compositor: CompositorState,
    layer_shell: LayerShell,
    layer_surface: Option<LayerSurface>, // TODO: support multiple outputs
    pointer: Option<wl_pointer::WlPointer>,

    shm_state: Shm,
    pool: SlotPool,
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,

    pub should_exit: bool,
    width: u32,
    height: u32,
    default_width: u32,
    default_height: u32,
    redraw: bool,
    widgets: Vec<Box<dyn Widget>>,
    last_moved_in: Option<usize>,
    last_damage: Vec<Rect>,
    lc: LC,
}

impl App {
    pub fn new(args: crate::Args) -> (Self, EventQueue<Self>) {
        let lc = LC::new("App", true);
        info!(lc, "| new :: Starting wayland client");
        let connection = Connection::connect_to_env().unwrap();

        let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();
        let qh = event_queue.handle();

        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");

        let surface = compositor.create_surface(&qh);
        let layer_surface =
            layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("wlrs-bar"), None);

        layer_surface.set_anchor(Anchor::BOTTOM.complement()); // anchor to all sides but the bottom
        layer_surface.set_size(args.width, args.height);
        layer_surface.set_exclusive_zone(args.height.try_into().unwrap());
        layer_surface.commit();

        let shm_state = Shm::bind(&globals, &qh).expect("wl_shm not available");

        let pool =
            SlotPool::new(4000 * args.height as usize, &shm_state).expect("Failed to create pool");
        //                ^^^^ seems like a reasonable default, 4, 1000 size buffers

        let font: rusttype::Font<'static> = args
            .font_path
            .and_then(|ref path| {
                std::fs::read(path)
                    .inspect_err(|err| warn!(lc, "| new :: failed to load custom font. {err}"))
                    .ok()
            })
            .and_then(|data| {
                let f = rusttype::Font::try_from_vec_and_index(data.to_vec(), args.font_index);
                if f.is_none() {
                    warn!(lc, "| new :: failed to initialize custom font.");
                }
                f
            })
            .unwrap_or_else(|| {
                rusttype::Font::try_from_bytes_and_index(DEFAULT_FONT_DATA, DEFAULT_FONT_INDEX)
                    .expect("app :: built-in font failed to initialize")
            });

        let mut widgets: Vec<Box<dyn Widget>> = Vec::new();

        #[cfg(feature = "clock")]
        widgets.push(Box::new(
            crate::clock::Clock::builder()
                .font(font.clone())
                .number_fg(color::ROSE)
                .spacer_fg(color::PINE)
                .bg(color::SURFACE)
                .desired_height(args.height)
                .build(LC::new("Clock", cfg!(feature = "clock-logs"))),
        ));

        #[cfg(feature = "workspaces")]
        match crate::workspaces::Workspaces::builder()
            .font(font.clone())
            .desired_height(args.height)
            .h_align(Align::Start)
            .fg(color::ROSE)
            .bg(color::SURFACE)
            .active_fg(color::ROSE)
            .active_bg(color::PINE)
            .hover_fg(color::GOLD)
            .hover_bg(color::H_MED)
            .build(LC::new("Workspaces", cfg!(feature = "workspaces-logs")))
        {
            Ok(w) => widgets.push(Box::new(w)),
            Err(err) => warn!(lc, "| new :: Workspaces failed to initialize. error={err}"),
        };

        #[cfg(any(
            feature = "battery",
            feature = "updated-last",
            feature = "cpu",
            feature = "ram",
            feature = "volume"
        ))]
        {
            let mut right_container = crate::widget::container::Container::builder()
                .h_align(Align::End)
                .inner_h_align(Align::End);

            #[cfg(feature = "updated-last")]
            if let Some(time_stamp) = args.updated_last {
                right_container.add(Box::new(
                    crate::updated_last::UpdatedLast::builder()
                        .font(font.clone())
                        .time_stamp(time_stamp)
                        .h_align(Align::End)
                        .fg(color::ROSE)
                        .bg(color::SURFACE)
                        .desired_height(args.height)
                        .build(LC::new("Updated Last", cfg!(feature = "updated-last-logs"))),
                ));
            } else {
                warn!(lc, "| new :: Updated Last not starting, no time_stamp provided, use '--updated-last <TIME_STAMP>'");
            }

            #[cfg(feature = "battery")]
            match crate::battery::Battery::builder()
                .font(font.clone())
                .battery_path(args.battery_path)
                .bg(color::SURFACE)
                .full_color(color::FOAM)
                .normal_color(color::PINE)
                .charging_color(color::GOLD)
                .warn_color(color::LOVE)
                .critical_color(color::LOVE)
                .desired_height(args.height)
                .desired_width(args.height)
                .h_align(Align::End)
                .build(LC::new("Battery", cfg!(feature = "battery-logs")))
            {
                Ok(w) => {
                    right_container.add(Box::new(w));
                }
                Err(err) => warn!(lc, "| new :: Battery widget disabled. error={err}"),
            }

            #[cfg(feature = "volume")]
            match crate::volume::Volume::builder()
                .font(font.clone())
                .fg(color::LOVE)
                .bg(color::SURFACE)
                .bar_filled(color::PINE)
                .desired_height(args.height)
                .build(LC::new("Volume", cfg!(feature = "volume-logs")))
            {
                Ok(w) => {
                    right_container.add(Box::new(w));
                }
                Err(err) => warn!(lc, "| new :: Volume widget disabled. error={err}"),
            }

            #[cfg(feature = "cpu")]
            match crate::cpu::Cpu::builder()
                .font(font.clone())
                .fg(color::LOVE)
                .bg(color::SURFACE)
                .bar_filled(color::PINE)
                .show_threshold(75.0)
                .desired_height(args.height)
                .build(LC::new("CPU", cfg!(feature = "cpu-logs")))
            {
                Ok(w) => {
                    right_container.add(Box::new(w));
                }
                Err(err) => warn!(lc, "| new :: CPU widget disabled. error={err}"),
            }

            #[cfg(feature = "ram")]
            match crate::ram::Ram::builder()
                .font(font.clone())
                .fg(color::LOVE)
                .bg(color::SURFACE)
                .bar_filled(color::PINE)
                .show_threshold(75.0)
                .desired_height(args.height)
                .build(LC::new("RAM", cfg!(feature = "ram-logs")))
            {
                Ok(w) => {
                    right_container.add(Box::new(w));
                }
                Err(err) => warn!(lc, "| new :: RAM widget disabled. error={err}"),
            }

            widgets.push(Box::new(
                right_container.build(LC::new("Right Container", false)),
            ));
        }

        let mut me = Self {
            //connection,
            compositor,
            layer_shell,
            layer_surface: Some(layer_surface),
            widgets,
            pointer: None,

            shm_state,
            pool,
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),

            width: args.width,
            height: args.height,
            default_width: args.width,
            default_height: args.height,

            redraw: true,
            last_damage: Vec::with_capacity(16),
            last_moved_in: None,
            should_exit: false,
            lc,
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
        new_factor: i32,
    ) {
        info!(
            self.lc,
            "| scale_factor_changed :: new scale factor (ignored) {new_factor:?}"
        );
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        new_transform: wl_output::Transform,
    ) {
        info!(
            self.lc,
            "| transform_changed :: New transform (ignored) {new_transform:?}"
        );
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

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        info!(self.lc, "| surface_enter :: surface entered");
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        info!(self.lc, "| surface_leave :: surface left");
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info!(self.lc, "| new_output :: a new output was added");

        if self.layer_surface.is_none() {
            info!(
                self.lc,
                "| new_output :: no current surface, making a new one on the output"
            );
            let surface = self.compositor.create_surface(qh);

            let layer_surface = self.layer_shell.create_layer_surface(
                &qh,
                surface,
                Layer::Top,
                Some("wlrs-bar"),
                None,
            );

            layer_surface.set_anchor(Anchor::BOTTOM.complement()); // anchor to all sides but the bottom
            layer_surface.set_size(self.default_width, self.default_height);
            layer_surface.set_exclusive_zone(self.default_height.try_into().unwrap());
            layer_surface.commit();

            self.layer_surface = Some(layer_surface);
        }
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info!(self.lc, "| update_output :: a output was updated (ignored)");
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info!(
            self.lc,
            "| output_destroyed :: a output was destroyed (ignored)"
        );
    }
}

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        if self.layer_surface.as_ref().is_some_and(|l| *l == *layer) {
            info!(self.lc, "| closed :: closing current surface.");
            self.layer_surface = None;
        } else {
            info!(self.lc, "| closed :: surface closed, that we didn't store?");
        }
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
            self.width = self.default_width; // let's hope this never recurses endlessly
            self.height = self.default_height;
        } else {
            debug!(
                self.lc,
                "| configure :: new size requested ({}, {})",
                configure.new_size.0,
                configure.new_size.1
            );
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }

        let (width, height) = (self.width, self.height);
        let canvas_size = Point {
            x: width,
            y: height,
        };
        let canvas = canvas_size.extend_to(Point::ZERO);

        for w in self.widgets.iter_mut() {
            let wid_height = w.desired_height().clamp(0, height);
            let wid_width = w.desired_width(wid_height).clamp(0, width);

            let size = Point {
                x: wid_width,
                y: wid_height,
            };
            trace!(self.lc, "| configure :: {} size: {size}", w.lc());

            let area = canvas.place_at(size, w.h_align(), w.v_align());
            trace!(self.lc, "| configure :: {} resized: {area}", w.lc());
            w.resize(area);
        }

        self.redraw = true;
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

    fn new_seat(&mut self, _conn: &Connection, _dh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        info!(self.lc, "| new_seat :: a new seat was added (ignored).");
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            debug!(self.lc, "| new_capability :: Set pointer capability");
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
            debug!(self.lc, "| new_capability :: Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        info!(self.lc, "| new_seat :: a new seat was added (ignored).");
    }
}

impl PointerHandler for App {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let point: Point = event.position.into();
            // Ignore events for other surfaces

            if self.layer_surface.is_none()
                || self
                    .layer_surface
                    .as_ref()
                    .is_some_and(|l| *l.wl_surface() != event.surface)
            {
                trace!(
                    self.lc,
                    "| pointer_frame :: got a click from another surface"
                );
                continue;
            }
            use PointerEventKind as PEK;

            match event.kind {
                PEK::Enter { .. } => {
                    assert!(self.last_moved_in.is_none());
                    if let Some((idx, w)) = self
                        .widgets
                        .iter_mut()
                        .enumerate()
                        .find(|(_idx, w)| w.area().contains(point))
                    {
                        if let Err(err) = w.motion(point) {
                            warn!(
                                self.lc,
                                "| pointer_frame :: widget {} motion failed. error={err}",
                                w.lc()
                            );
                        }
                        self.last_moved_in = Some(idx);
                    }
                }
                PEK::Leave { .. } => {
                    if let Some(w) = self.last_moved_in.and_then(|idx| self.widgets.get_mut(idx)) {
                        trace!(self.lc, "| pointer_frame :: left widget {}", w.lc());
                        if let Err(err) = w.motion_leave(point) {
                            warn!(
                                self.lc,
                                "| pointer_frame :: widget {} motion_leave failed. error={err}",
                                w.lc()
                            );
                        }
                    }
                    self.last_moved_in = None;
                }
                PEK::Motion { .. } => {
                    let moved_in_idx = self
                        .widgets
                        .iter_mut()
                        .enumerate()
                        .find(|(_idx, w)| w.area().contains(point))
                        .map(|(idx, w)| {
                            if let Err(err) = w.motion(point) {
                                warn!(
                                    self.lc,
                                    "| pointer_frame :: widget {} motion failed. error={err}",
                                    w.lc()
                                );
                            }
                            idx
                        });

                    if self.last_moved_in != moved_in_idx {
                        if let Some(w) =
                            self.last_moved_in.and_then(|idx| self.widgets.get_mut(idx))
                        {
                            trace!(self.lc, "| pointer_frame :: left widget {}", w.lc());
                            if let Err(err) = w.motion_leave(point) {
                                warn!(
                                    self.lc,
                                    "| pointer_frame :: widget {} motion_leave failed. error={err}",
                                    w.lc()
                                );
                            }
                        }
                    }
                    self.last_moved_in = moved_in_idx;
                }
                PEK::Press { .. } => {
                    // only care about releasing, not pressing
                    //trace!("pointer_frame :: Press {:x} @ {:?}", button, event.position);
                }
                PEK::Release { button, .. } => {
                    if let Some(widget) = self.widgets.iter_mut().find(|w| w.area().contains(point))
                    {
                        if let Err(err) = widget.click(ClickType::new(button), point) {
                            warn!(
                                self.lc,
                                "| pointer_frame :: click on {} failed. error={err}",
                                widget.lc()
                            );
                        }
                    }
                }
                PEK::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    trace!(
                        self.lc,
                        "pointer_frame :: Scroll H:{horizontal:?}, V:{vertical:?}"
                    );
                }
            }
        }
    }
}

impl App {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let layer = match &self.layer_surface {
            Some(l) => l,
            None => return, // nothing to draw onto.
        };
        let surface = layer.wl_surface();

        //self.pool
        //    .resize((self.width * self.height * 4) as usize)
        //    .unwrap();
        let stride: i32 = i32::try_from(self.width).unwrap() * 4;

        // TODO: Reuse these buffers :)
        let (buffer, canvas) = self
            .pool
            .create_buffer(
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
                stride,
                wl_shm::Format::Argb8888,
            )
            .unwrap();

        let rect = Point::ZERO.extend_to(Point {
            x: self.width,
            y: self.height,
        });

        if cfg!(feature = "damage") {
            let mut ctx = crate::draw::DrawCtx {
                damage: &mut Vec::new(),
                buffer: &buffer,
                canvas,
                rect,
                full_redraw: self.redraw,
            };

            for dam in self.last_damage.iter() {
                dam.draw_outline(color::SURFACE, &mut ctx);
                dam.damage_outline(&surface);
            }
        }

        let mut ctx = crate::draw::DrawCtx {
            damage: &mut self.last_damage,
            buffer: &buffer,
            canvas,
            rect,
            full_redraw: self.redraw,
        };

        ctx.damage.clear();

        if self.redraw {
            debug!(self.lc, "| draw :: full redraw");
            rect.draw(color::SURFACE, &mut ctx);
        }

        for w in self.widgets.iter_mut() {
            if w.should_redraw() {
                if let Err(err) = w.draw(&mut ctx) {
                    warn!(
                        self.lc,
                        "| draw :: widget {} failed to draw: error={err}",
                        w.lc()
                    );
                }
            }
            #[cfg(feature = "outlines")]
            w.area().draw_outline(color::PINE, &mut ctx);
        }

        if self.redraw {
            self.redraw = false;

            // Damage the entire window
            surface.damage_buffer(
                0,
                0,
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
            );
            ctx.damage.clear();
        } else {
            let damage = ctx.damage.clone();
            for dam in damage {
                surface.damage_buffer(
                    dam.min.x.try_into().unwrap(),
                    dam.min.y.try_into().unwrap(),
                    dam.max.x.try_into().unwrap(),
                    dam.max.y.try_into().unwrap(),
                );

                #[cfg(feature = "damage")]
                dam.draw_outline(color::LOVE, &mut ctx);
            }
        }

        surface.frame(qh, surface.clone()); // Request our next frame
        ctx.buffer.attach_to(surface).unwrap();

        layer.commit();

        if cfg!(feature = "height-test") {
            // hack to test all sizes above your own (until it hits some limit)
            info!(self.lc, "| draw :: height: {}", self.height);
            layer.set_size(self.default_width, self.height - 1);
            layer.set_exclusive_zone(self.height as i32 - 1);
            layer.commit();
        }
    }

    pub fn run_queue(&mut self, event_queue: &mut EventQueue<Self>) {
        loop {
            if let Err(err) = event_queue.blocking_dispatch(self) {
                warn!(self.lc, "| run_queue :: event queue error: error={err}");
            }

            if self.should_exit {
                info!(self.lc, "| run_queue :: exiting...");
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
