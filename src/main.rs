use std::{borrow::Borrow, cell::RefCell, ptr::NonNull, rc::Rc};

use egui::{Context, Ui};
use miniquad::conf;
use mlua::{FromLua, IntoLua, Lua, UserData, UserDataMethods, Value};
use system::Vapo;

mod system {

    use miniquad::{self as mq, conf};

    use crate::GUIData;

    pub struct Vapo {
        egui_mq: egui_miniquad::EguiMq,
        mq_ctx: Box<dyn mq::RenderingBackend>,
        gui: GUIData,
    }

    impl Vapo {
        pub fn new(gui: GUIData) -> Self {
            let mut mq_ctx = mq::window::new_rendering_backend();
            Self {
                egui_mq: egui_miniquad::EguiMq::new(&mut *mq_ctx),
                mq_ctx,
                gui,
            }
        }
    }

    impl mq::EventHandler for Vapo {
        fn update(&mut self) {}

        fn draw(&mut self) {
            self.mq_ctx
                .begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
            self.mq_ctx.end_render_pass();

            self.egui_mq.run(&mut *self.mq_ctx, |_mq_ctx, egui_ctx| {
                self.gui.main_window(egui_ctx)
            });

            // Draw things behind egui here

            self.egui_mq.draw(&mut *self.mq_ctx);

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
}

struct GUIData {
    lua: Lua,
    ui: Option<NonNull<Ui>>,
}

impl GUIData {
    pub fn main_window(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            self.ui = Some(NonNull::from(ui));
            self.draw();
            self.ui = None;
        });
    }
    pub fn ui(&self) -> &mut Ui {
        unsafe { self.ui.unwrap().as_mut() }
    }
    pub fn draw(&self) {}
}

impl UserData for GUIData {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("label", |lua, this, (label,): (Value,)| {});
    }
}
#[derive(Clone, Default)]
struct Ref<T> {
    data: Rc<RefCell<T>>,
}

impl<T> Ref<T> {
    pub fn new(val: T) -> Self {
        Self {
            data: Rc::new(RefCell::new(val)),
        }
    }
    pub fn ref_(&self) -> std::cell::Ref<T> {
        (*self.data).borrow()
    }

    pub fn mut_(&self) -> std::cell::RefMut<T> {
        (*self.data).borrow_mut()
    }
}

type StrRef = Ref<String>;

trait AsLua {
    fn into_lua(&self, lua: &Lua) -> mlua::Result<Value>;
}

trait DualLua: FromLua + AsLua {}
impl<T> DualLua for T where T: FromLua + AsLua {}

impl AsLua for String {
    fn into_lua(&self, lua: &Lua) -> mlua::Result<Value> {
        self.as_str().into_lua(lua)
    }
}

impl<T> UserData for Ref<T>
where
    T: DualLua,
{
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |lua, this, ()| this.ref_().into_lua(lua));
        methods.add_method("set", |lua, this, (value,)| {
            *this.mut_() = T::from_lua(value, lua)?;
            Ok(())
        });
    }
}

fn init_lua(lua: &mut Lua) -> Result<(), mlua::Error> {
    let vapo = lua.create_table()?;
    vapo.set(
        "dstr",
        lua.create_function(|lua, ()| Ok(StrRef::default()))?,
    )?;
    lua.globals().set("vapo", vapo)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conf = miniquad::conf::Conf::default();
    let mut lua = Lua::new();
    init_lua(&mut lua)?;
    let source = std::fs::read_to_string("vapo.lua")?;
    conf.platform.apple_gfx_api = conf::AppleGfxApi::Metal;
    // conf.platform.linux_backend = LinuxBackend::WaylandOnly;
    miniquad::start(conf, move || Box::new(Vapo::new(GUIData { lua, ui: None })));
    Ok(())
}
