use futures::executor;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, HasRawDisplayHandle, HasRawWindowHandle,
    RawDisplayHandle, RawWindowHandle,
};
use std::ffi;

mod render;

pub use render::render::State;

#[no_mangle]
pub extern "C" fn hello_world() {
    println!("Hello World!");
}

#[no_mangle]
pub extern "C" fn send_window(appKitNSView: *mut ffi::c_void) {
    let window = Window {
        ns_view: appKitNSView,
    };
    /*let mut window_handle = AppKitWindowHandle::empty();
    window_handle.ns_view = appKitNSView;

    let handle = RawWindowHandle::AppKit(window_handle);*/

    let mut res = State::new(&window);
    let mut v = executor::block_on(res);

    v.render();
}

pub fn send_raw_window<
    W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
>(
    window: &W,
) {
    let mut res = State::new(&window);
    let mut v = executor::block_on(res);

    v.render();
}

pub struct Window {
    //id: usize,
    // ns_window: *mut ffi::c_void,
    ns_view: *mut ffi::c_void,
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::AppKit(AppKitDisplayHandle::empty())
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AppKitWindowHandle::empty();
        // handle.ns_window = self.ns_window;
        handle.ns_view = self.ns_view;
        RawWindowHandle::AppKit(handle)
    }
}
