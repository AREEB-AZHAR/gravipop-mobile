pub mod core;
pub mod physics;
pub mod graphics;
pub mod audio;
pub mod monetization;
pub mod ui;

// JNI Entry point for Android NDK
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main(app: *mut std::ffi::c_void) {
    // Android native activity entry point handled by miniquad
}
