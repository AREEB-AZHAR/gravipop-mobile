#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn gravipop_ads_ready(rewarded: i32) -> i32;
    fn gravipop_ads_show(request_id: u32, rewarded: i32) -> i32;
    fn gravipop_ads_poll(request_id: u32) -> i32;
    fn gravipop_ads_set_removed(removed: i32);
}

pub fn is_ready(rewarded: bool) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { gravipop_ads_ready(i32::from(rewarded)) == 1 }
    }
    #[cfg(target_os = "android")]
    {
        android::is_ready(rewarded)
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        let _ = rewarded;
        false
    }
}

pub fn show(request_id: u32, rewarded: bool) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { gravipop_ads_show(request_id, i32::from(rewarded)) == 1 }
    }
    #[cfg(target_os = "android")]
    {
        android::show(request_id, rewarded)
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        let _ = (request_id, rewarded);
        false
    }
}

pub fn poll(request_id: u32) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { gravipop_ads_poll(request_id) }
    }
    #[cfg(target_os = "android")]
    {
        android::poll(request_id)
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        let _ = request_id;
        0
    }
}

pub fn set_removed(removed: bool) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        gravipop_ads_set_removed(i32::from(removed));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = removed;
    }
}

pub fn privacy_options_required() -> bool {
    #[cfg(target_os = "android")]
    {
        android::privacy_options_required()
    }
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}

pub fn show_privacy_options() {
    #[cfg(target_os = "android")]
    android::show_privacy_options();
}

#[cfg(target_os = "android")]
mod android {
    use macroquad::miniquad::native::android::{self, ndk_sys::*};
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Mutex,
    };

    static AVAILABILITY: AtomicU8 = AtomicU8::new(0);
    static RESULTS: Mutex<Vec<(u32, i32)>> = Mutex::new(Vec::new());

    pub fn is_ready(rewarded: bool) -> bool {
        AVAILABILITY.load(Ordering::Acquire) & if rewarded { 1 } else { 2 } != 0
    }

    pub fn privacy_options_required() -> bool {
        AVAILABILITY.load(Ordering::Acquire) & 4 != 0
    }

    // Miniquad already attaches the render thread to the JVM. All Kotlin SDK
    // operations are dispatched to the UI thread by the Activity methods.
    unsafe fn call_boolean(method_name: &[u8], signature: &[u8], args: &[jvalue]) -> bool {
        let activity = android::ACTIVITY;
        if activity.is_null() {
            return false;
        }
        let env = android::attach_jni_env();
        let class = (**env).GetObjectClass.unwrap()(env, activity);
        if class.is_null() {
            return false;
        }
        let method = (**env).GetMethodID.unwrap()(
            env,
            class,
            method_name.as_ptr().cast(),
            signature.as_ptr().cast(),
        );
        if method.is_null() {
            (**env).ExceptionClear.unwrap()(env);
            (**env).DeleteLocalRef.unwrap()(env, class);
            return false;
        }
        let result = (**env).CallBooleanMethodA.unwrap()(env, activity, method, args.as_ptr());
        let failed = (**env).ExceptionCheck.unwrap()(env) != 0;
        if failed {
            (**env).ExceptionClear.unwrap()(env);
        }
        (**env).DeleteLocalRef.unwrap()(env, class);
        result != 0 && !failed
    }

    pub fn show(request_id: u32, rewarded: bool) -> bool {
        unsafe {
            call_boolean(
                b"requestAdFromNative\0",
                b"(IZ)Z\0",
                &[
                    jvalue {
                        i: request_id as jint,
                    },
                    jvalue {
                        z: u8::from(rewarded),
                    },
                ],
            )
        }
    }

    pub fn show_privacy_options() {
        unsafe {
            call_boolean(b"showAdPrivacyOptionsFromNative\0", b"()Z\0", &[]);
        }
    }

    pub fn poll(request_id: u32) -> i32 {
        let mut results = RESULTS.lock().unwrap_or_else(|e| e.into_inner());
        let result = results
            .iter()
            .find(|(id, _)| *id == request_id)
            .map_or(0, |(_, status)| *status);
        results.clear();
        result
    }

    #[no_mangle]
    pub unsafe extern "system" fn Java_com_gravipop_celestialmerge_MainActivity_onNativeAdAvailability(
        _env: *mut JNIEnv,
        _activity: jobject,
        flags: jint,
    ) {
        AVAILABILITY.store(flags as u8, Ordering::Release);
    }

    #[no_mangle]
    pub unsafe extern "system" fn Java_com_gravipop_celestialmerge_MainActivity_onNativeAdFinished(
        _env: *mut JNIEnv,
        _activity: jobject,
        request_id: jint,
        status: jint,
    ) {
        let mut results = RESULTS.lock().unwrap_or_else(|e| e.into_inner());
        results.push((request_id as u32, status));
    }
}
