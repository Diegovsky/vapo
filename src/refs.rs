use std::{cell::RefCell, ops::Deref, rc::Rc};

use mlua::prelude::*;

#[derive(Default)]
pub struct Ref<T> {
    data: Rc<RefCell<T>>,
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
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

trait AsLua {
    fn as_lua(&self, lua: &Lua) -> mlua::Result<LuaValue>;
}

trait DualLua: FromLua + AsLua {}
impl<T> DualLua for T where T: FromLua + AsLua {}

trait RefExt: Sized + DualLua {
    fn add_methods<M: LuaUserDataMethods<Ref<Self>>>(methods: &mut M) {
        let _ = methods;
    }
}

impl<T> LuaUserData for Ref<T>
where
    T: DualLua + RefExt,
{
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |lua, this, ()| this.ref_().as_lua(lua));
        methods.add_method("set", |lua, this, (value,)| {
            dbg!(&value);
            *this.mut_() = T::from_lua(value, lua)?;
            Ok(())
        });
        T::add_methods(methods);
    }
}

impl<T> FromLua for Ref<T>
where
    T: DualLua + 'static,
{
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let err = |msg| LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: std::any::type_name::<Self>().into(),
            message: msg,
        };
        let value = value
            .as_userdata()
            .ok_or_else(|| err(None))?
            .borrow_scoped::<Self, _>(|t| t.clone())?;
        Ok(value)
    }
}

// Implementation of a String `ref`
pub(crate) type StrRef = Ref<String>;

impl AsLua for String {
    fn as_lua(&self, lua: &Lua) -> mlua::Result<LuaValue> {
        self.as_str().into_lua(lua)
    }
}

impl RefExt for String {
    fn add_methods<M: LuaUserDataMethods<Ref<Self>>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            this.ref_().as_lua(lua)
        });
    }
}
