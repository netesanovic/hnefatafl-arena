use crate::bot::Bot;
use crate::game::{GameState, Move, Player};
use libloading::{Library, Symbol};
use std::path::Path;
use std::time::Duration;

/// FFI-safe representation of a bot plugin
/// This is the interface used to load bots from dynamic libraries
#[repr(C)]
pub struct BotPlugin {
    pub bot_ptr: *mut (),
    pub vtable: BotVTable,
}

/// Virtual table for bot operations
/// All bot implementations must provide these function pointers
#[repr(C)]
pub struct BotVTable {
    pub name: unsafe extern "C" fn(*mut ()) -> *const std::os::raw::c_char,
    pub get_move: unsafe extern "C" fn(*mut (), *const GameState, u64) -> *const Move,
    pub game_start: unsafe extern "C" fn(*mut (), Player),
    pub notify_move: unsafe extern "C" fn(*mut (), Move),
    pub game_end: unsafe extern "C" fn(*mut ()),
    pub drop: unsafe extern "C" fn(*mut ()),
}

/// Type signature for the plugin creation function
/// Every plugin library must export a function with this signature
pub type CreateBotFn = unsafe extern "C" fn() -> *mut BotPlugin;

/// Wrapper that loads a bot from a dynamic library
pub struct PluginBot {
    plugin: Box<BotPlugin>,
    _library: Library, // Keep library alive
}

impl PluginBot {
    /// Load a bot plugin from a dynamic library file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        unsafe {
            let library = Library::new(path.as_ref())
                .map_err(|e| format!("Failed to load library: {}", e))?;

            let create_bot: Symbol<CreateBotFn> = library
                .get(b"create_bot")
                .map_err(|e| format!("Failed to find create_bot function: {}", e))?;

            let plugin_ptr = create_bot();
            if plugin_ptr.is_null() {
                return Err("create_bot returned null".to_string());
            }

            let plugin = Box::from_raw(plugin_ptr);

            Ok(PluginBot {
                plugin,
                _library: library,
            })
        }
    }
}

impl Bot for PluginBot {
    fn name(&self) -> &str {
        unsafe {
            let name_ptr = (self.plugin.vtable.name)(self.plugin.bot_ptr);
            if name_ptr.is_null() {
                return "Unknown";
            }
            let c_str = std::ffi::CStr::from_ptr(name_ptr);
            c_str.to_str().unwrap_or("Invalid UTF-8")
        }
    }

    fn get_move(&mut self, state: &GameState, time_limit: Duration) -> Option<Move> {
        unsafe {
            let move_ptr = (self.plugin.vtable.get_move)(
                self.plugin.bot_ptr,
                state as *const GameState,
                time_limit.as_millis() as u64,
            );

            if move_ptr.is_null() {
                None
            } else {
                Some(*move_ptr)
            }
        }
    }

    fn game_start(&mut self, player: Player) {
        unsafe {
            (self.plugin.vtable.game_start)(self.plugin.bot_ptr, player);
        }
    }

    fn notify_move(&mut self, mv: Move) {
        unsafe {
            (self.plugin.vtable.notify_move)(self.plugin.bot_ptr, mv);
        }
    }

    fn game_end(&mut self) {
        unsafe {
            (self.plugin.vtable.game_end)(self.plugin.bot_ptr);
        }
    }
}

impl Drop for PluginBot {
    fn drop(&mut self) {
        unsafe {
            (self.plugin.vtable.drop)(self.plugin.bot_ptr);
        }
    }
}

unsafe impl Send for PluginBot {}

/// Helper macro for implementing a bot plugin
/// This handles all the FFI boilerplate
#[macro_export]
macro_rules! export_bot {
    ($bot_type:ty) => {
        use std::ffi::CString;
        use std::os::raw::c_char;

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn create_bot() -> *mut $crate::plugin::BotPlugin {
            let bot = Box::new(<$bot_type>::default());
            let bot_ptr = Box::into_raw(bot) as *mut ();

            let vtable = $crate::plugin::BotVTable {
                name: bot_name,
                get_move: bot_get_move,
                game_start: bot_game_start,
                notify_move: bot_notify_move,
                game_end: bot_game_end,
                drop: bot_drop,
            };

            Box::into_raw(Box::new($crate::plugin::BotPlugin { bot_ptr, vtable }))
        }

        unsafe extern "C" fn bot_name(ptr: *mut ()) -> *const c_char {
            let bot = &*(ptr as *const $bot_type);
            let name = bot.name();
            let c_string = CString::new(name).unwrap();
            c_string.into_raw()
        }

        unsafe extern "C" fn bot_get_move(
            ptr: *mut (),
            state: *const $crate::game::GameState,
            time_limit_ms: u64,
        ) -> *const $crate::game::Move {
            let bot = &mut *(ptr as *mut $bot_type);
            let state = &*state;
            let time_limit = std::time::Duration::from_millis(time_limit_ms);

            match bot.get_move(state, time_limit) {
                Some(mv) => Box::into_raw(Box::new(mv)),
                None => std::ptr::null(),
            }
        }

        unsafe extern "C" fn bot_game_start(ptr: *mut (), player: $crate::game::Player) {
            let bot = &mut *(ptr as *mut $bot_type);
            bot.game_start(player);
        }

        unsafe extern "C" fn bot_notify_move(ptr: *mut (), mv: $crate::game::Move) {
            let bot = &mut *(ptr as *mut $bot_type);
            bot.notify_move(mv);
        }

        unsafe extern "C" fn bot_game_end(ptr: *mut ()) {
            let bot = &mut *(ptr as *mut $bot_type);
            bot.game_end();
        }

        unsafe extern "C" fn bot_drop(ptr: *mut ()) {
            let _ = Box::from_raw(ptr as *mut $bot_type);
        }
    };
}
