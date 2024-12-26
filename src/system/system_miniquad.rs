use miniquad::{
    self as mq,
    conf::{self, Conf},
};

use crate::Vapo;

pub struct System {
    egui_mq: egui_miniquad::EguiMq,
    mq_ctx: Box<dyn mq::RenderingBackend>,
    pub(crate) vapo: Vapo,
}

impl System {
    pub fn new(vapo: Vapo) -> Self {
        let mut mq_ctx = mq::window::new_rendering_backend();
        Self {
            egui_mq: egui_miniquad::EguiMq::new(&mut *mq_ctx),
            mq_ctx,
            vapo,
        }
    }
}

impl mq::EventHandler for System {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.mq_ctx
            .begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        self.mq_ctx.end_render_pass();

        self.egui_mq.run(&mut *self.mq_ctx, |_mq_ctx, egui_ctx| {
            self.vapo.main_window(egui_ctx)
        });

        // Draw things behind egui here

        self.egui_mq.draw(&mut *self.mq_ctx);
        if self.vapo.should_close {
            miniquad::window::request_quit();
        }

        // Draw things in front of egui here

        self.mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
    }

    fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.key_down_event(keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

pub fn run(vapo: Vapo) {
    let mut conf = Conf::default();
    conf.platform.apple_gfx_api = conf::AppleGfxApi::Metal;
    // conf.platform.linux_backend = LinuxBackend::WaylandOnly;
    miniquad::start(conf, move || Box::new(System::new(vapo)));
}
