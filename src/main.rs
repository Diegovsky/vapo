use std::ptr::NonNull;

use egui::{Color32, Context, Ui};
use mlua::prelude::*;
use mlua::{Lua, ObjectLike, Table, UserData, UserDataMethods, Value};
use refs::StrRef;

pub struct Vapo {
    pub lua: Lua,
    pub gui: GUIData,
    pub error: Option<String>,
    pub should_close: bool,
}

impl Vapo {
    pub(crate) fn main_window(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            if let Some(ref err) = self.error {
                ui.colored_label(
                    Color32::from_rgb(0xaa, 0x66, 0x66),
                    "An Error has Occoured!",
                );
                ui.label(err);
                if ui.button("Quit").clicked() {
                    self.request_quit();
                }
            } else {
                self.gui.ui = Some(NonNull::from(ui));
                if let Err(e) = self.draw() {
                    eprintln!("Error: {e}");
                    self.error = Some(e.to_string());
                }
                self.gui.ui = None;
            }
        });
    }

    pub(crate) fn draw(&mut self) -> mlua::Result<()> {
        self.lua.scope(|scope| {
            let ui = scope.create_userdata_ref_mut(&mut self.gui)?;
            self.lua
                .globals()
                .get::<Table>("vapo")?
                .call_function::<()>("draw", (ui,))?;

            Ok(())
        })?;
        Ok(())
    }

    pub fn request_quit(&mut self) {
        self.should_close = true;
    }
}

mod system;

#[derive(Default)]
pub struct GUIData {
    ui: Option<NonNull<Ui>>,
}

impl GUIData {
    pub fn ui(&self) -> &mut Ui {
        unsafe {
            self.ui
                .expect("Called ui function out of context.")
                .as_mut()
        }
    }
    fn get_string<T>(
        &self,
        lua: &Lua,
        value: Value,
        access: impl FnOnce(&str) -> T,
    ) -> mlua::Result<T> {
        Ok(if let Some(value) = lua.coerce_string(value)? {
            access(&*value.to_str()?)
        } else {
            access("[Unknown]")
        })
    }
    fn lua_label(&self, lua: &Lua, (label,): (Value,)) -> mlua::Result<()> {
        let ui = self.ui();
        self.get_string(lua, label, |label| ui.label(label))?;
        Ok(())
    }

    fn lua_input(&self, _lua: &Lua, (sref,): (StrRef,)) -> mlua::Result<()> {
        self.ui().text_edit_singleline(&mut *sref.mut_());
        Ok(())
    }
    fn lua_button(&self, lua: &Lua, (label,): (Value,)) -> mlua::Result<bool> {
        let clicked = self
            .get_string(lua, label, |label| self.ui().button(label))?
            .clicked();
        Ok(clicked)
    }
}

impl UserData for GUIData {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        macro_rules! method {
            ($name:expr, $ident:ident) => {
                methods.add_method($name, |lua, this, args| Self::$ident(this, lua, args))
            };
            (mut $name:expr, $ident:ident) => {
                methods.add_method_mut($name, |lua, this, args| Self::$ident(this, lua, args))
            };
        }
        method!("label", lua_label);
        method!("input", lua_input);
        method!("button", lua_button);
    }
}

mod refs;

fn init_lua(lua: &mut Lua) -> Result<(), mlua::Error> {
    let vapo = lua.create_table()?;
    vapo.set(
        "dstr",
        lua.create_function(|_lua, ()| Ok(StrRef::default()))?,
    )?;
    lua.globals().set("vapo", vapo)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lua = Lua::new();
    init_lua(&mut lua)?;
    let source = std::fs::read_to_string("vapo.lua")?;
    lua.load(source).set_name("vapo.lua").exec()?;
    let vapo = Vapo {
        lua,
        error: None,
        should_close: false,
        gui: GUIData::default(),
    };
    system::run(vapo);
    Ok(())
}
