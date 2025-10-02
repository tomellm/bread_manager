use mlua::{Function, Lua, LuaSerdeExt};

use crate::model::transactions::Transaction;

pub struct CodeFilter {
    lua: Lua,
    func: Function,
}

impl CodeFilter {
    pub fn from_code(code: &str) -> Self {
        let lua = Lua::new();
        let func: Function = lua
            .load(code)
            .eval()
            .map_err(|err| {
                tracing::error!(
                    "error while loading the filter function: {err:?}"
                );
            })
            .unwrap();
        Self { lua, func }
    }

    pub fn call(&self, trxn: &Transaction) -> Result<bool, mlua::Error> {
        let val_trxn = self.lua.to_value(trxn).unwrap();
        self.func.call::<bool>(val_trxn)
    }
}
