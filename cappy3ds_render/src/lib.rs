use bytes::BytesMut;
use futures::executor;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, HasRawDisplayHandle, HasRawWindowHandle,
    RawDisplayHandle, RawWindowHandle, WindowsDisplayHandle,
};
use std::ffi;
use wgpu_hal;

mod dsscreen;
mod primitive;
mod render;

use std::thread;

pub use render::State;

#[no_mangle]
#[cfg(target_os = "windows")]
pub extern "C" fn send_visual(callback: unsafe extern "C" fn( *mut ffi::c_void)) {
    use wgpu::Surface;

    let mut res = State::new_from_visual();
    let mut v = executor::block_on(res);

    unsafe {
        v.surface
            .as_hal_mut::<wgpu_hal::dx12::Api, _, _>(|surface| {
                match surface {
                    Some(surface) => {
                        // bleh it's private
                        callback(surface.swap_chain.as_raw() as *mut ffi::c_void);
                    },
                    None => todo!(),
                }
            })
    };

    // v.render();
}

#[no_mangle]
#[cfg(target_os = "macos")]
pub extern "C" fn send_window(app_kit_nsview: *mut ffi::c_void) {
    let window = Window {
        ns_view: app_kit_nsview,
    };

    let res = State::new(&window);
    let v = executor::block_on(res);

    v.render();
}

pub fn send_raw_window<
    W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
>(
    window: &W,
) {
    //capture::do_capture();

    let res = State::new(&window);
    let v = executor::block_on(res);

    let mut heheh = Box::new(v);

    heheh.render();

    let thread_join_handle = thread::spawn(move || loop {
        trash_code(heheh.as_mut());
    });
}

fn trash_code(v: &mut State) {
    let mut cappy3ds = cappy3ds::Cappy3ds::new(
        |audio: &[i16], upper_buffer: BytesMut, lower_buffer: BytesMut| {
            if upper_buffer.len() >= 288000 {
                v.write_texture(&upper_buffer, &lower_buffer);

                v.render();
            }
        },
    );

    cappy3ds.connect();

    cappy3ds.do_capture();
}

#[cfg(target_os = "macos")]
pub struct Window {
    ns_view: *mut ffi::c_void,
}

#[cfg(target_os = "macos")]
unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::AppKit(AppKitDisplayHandle::empty())
    }
}

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AppKitWindowHandle::empty();
        // handle.ns_window = self.ns_window;
        handle.ns_view = self.ns_view;
        RawWindowHandle::AppKit(handle)
    }
}
