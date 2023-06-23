use cocoa::{
    base::{id, nil},
    foundation::NSAutoreleasePool,
};
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Sel},
    sel, sel_impl,
};
use once_cell::sync::OnceCell;

use crate::api::TheseusSerializableError;

type Callback = OnceCell<Box<dyn Fn(String) + Send + Sync + 'static>>;

static CALLBACK: Callback = OnceCell::new();

pub struct AppDelegateClass(pub *const Class);
unsafe impl Send for AppDelegateClass {}
unsafe impl Sync for AppDelegateClass {}

// Obj C class for the app delegate
// This inherits from the TaoAppDelegate (used by tauri) so we do not accidentally override any functionality
// The application_open_file method is the only method we override, as it is currently unimplemented in tauri
lazy_static::lazy_static! {
    pub static ref THESEUS_APP_DELEGATE_CLASS: AppDelegateClass = unsafe {
        let superclass = class!(TaoAppDelegate);
        let mut decl = ClassDecl::new("TheseusAppDelegate", superclass).unwrap();

        // Add the method to the class
        decl.add_method(
          sel!(application:openFile:),
          application_open_file as extern "C" fn(&Object, Sel, id, id) -> bool,
        );

        // Other methods are inherited

        AppDelegateClass(decl.register())
    };
}

extern "C" fn application_open_file(
    _: &Object,
    _: Sel,
    _: id,
    file: id,
) -> bool {
    let file = nsstring_to_string(file);
    callback(file)
}

pub fn callback(file: String) -> bool {
    if let Some(callback) = CALLBACK.get() {
        callback(file);
        true
    } else {
        false
    }
}

pub fn register_open_file<T>(
    callback: T,
) -> Result<(), TheseusSerializableError>
where
    T: Fn(String) + Send + Sync + 'static,
{
    unsafe {
        // Modified from tao: https://github.com/tauri-apps/tao
        // sets the current app delegate to be the inherited app delegate rather than the default tauri/tao one
        let app: id = msg_send![class!(TaoApp), sharedApplication];

        let delegate: id = msg_send![THESEUS_APP_DELEGATE_CLASS.0, new];
        let pool = NSAutoreleasePool::new(nil);
        let _: () = msg_send![app, setDelegate: delegate];
        let _: () = msg_send![pool, drain];
    }
    CALLBACK.set(Box::new(callback)).map_err(|_| {
        TheseusSerializableError::Callback("Callback already set".to_string())
    })
}

/// Convert an NSString to a Rust `String`
/// From 'fruitbasket' https://github.com/mrmekon/fruitbasket/
#[allow(clippy::cmp_null)]
pub fn nsstring_to_string(nsstring: *mut Object) -> String {
    unsafe {
        let cstr: *const i8 = msg_send![nsstring, UTF8String];
        if cstr != std::ptr::null() {
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            "".into()
        }
    }
}
