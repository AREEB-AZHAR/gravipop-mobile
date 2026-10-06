// GraviPop Lightweight Web FFI Bridge
// Replaces heavy wasm-bindgen and web-sys dependencies with direct, zero-overhead
// Miniquad JavaScript FFI calls. Fully compatible with Macroquad's runtime.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn gravipop_storage_len(key_ptr: *const u8, key_len: u32) -> i32;
    fn gravipop_storage_get(key_ptr: *const u8, key_len: u32, out_ptr: *mut u8, max_len: u32) -> i32;
    fn gravipop_storage_set(key_ptr: *const u8, key_len: u32, val_ptr: *const u8, val_len: u32);

    fn gravipop_sync_name_input(
        show: i32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        score_high: u32,
        score_low: u32,
        init_ptr: *const u8,
        init_len: u32,
        out_ptr: *mut u8,
        out_max_len: u32,
    ) -> i32;

    fn gravipop_leaderboard_refresh();
    fn gravipop_take_submit_request() -> i32;
    fn gravipop_leaderboard_submit(name_ptr: *const u8, name_len: u32, score_high: u32, score_low: u32);
    fn gravipop_leaderboard_status_len() -> i32;
    fn gravipop_leaderboard_status_get(out_ptr: *mut u8, max_len: u32) -> i32;
    fn gravipop_leaderboard_data_len() -> i32;
    fn gravipop_leaderboard_data_get(out_ptr: *mut u8, max_len: u32) -> i32;
    fn gravipop_leaderboard_configured() -> i32;
}

pub fn take_submit_request() -> bool {
    #[cfg(target_arch = "wasm32")]
    { unsafe { gravipop_take_submit_request() != 0 } }
    #[cfg(not(target_arch = "wasm32"))]
    { false }
}

pub fn storage_get(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let key_bytes = key.as_bytes();
        let len = unsafe { gravipop_storage_len(key_bytes.as_ptr(), key_bytes.len() as u32) };
        if len <= 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        let copied = unsafe {
            gravipop_storage_get(
                key_bytes.as_ptr(),
                key_bytes.len() as u32,
                buf.as_mut_ptr(),
                len as u32,
            )
        };
        if copied > 0 {
            buf.truncate(copied as usize);
            String::from_utf8(buf).ok()
        } else {
            None
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        None
    }
}

pub fn storage_set(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let key_bytes = key.as_bytes();
        let val_bytes = value.as_bytes();
        unsafe {
            gravipop_storage_set(
                key_bytes.as_ptr(),
                key_bytes.len() as u32,
                val_bytes.as_ptr(),
                val_bytes.len() as u32,
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, value);
    }
}

pub fn sync_name_input(
    show: bool,
    bounds: (f32, f32, f32, f32),
    current: &str,
    initialize: bool,
    score: u64,
) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let init_bytes = if initialize { current.as_bytes() } else { &[] };
        let mut buf = vec![0u8; 128];
        let score_high = (score >> 32) as u32;
        let score_low = (score & 0xffff_ffff) as u32;
        let len = unsafe {
            gravipop_sync_name_input(
                if show { 1 } else { 0 },
                bounds.0,
                bounds.1,
                bounds.2,
                bounds.3,
                score_high,
                score_low,
                init_bytes.as_ptr(),
                init_bytes.len() as u32,
                buf.as_mut_ptr(),
                buf.len() as u32,
            )
        };
        if show && len >= 0 {
            buf.truncate(len as usize);
            String::from_utf8(buf).ok()
        } else {
            None
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (show, bounds, current, initialize, score);
        None
    }
}

pub fn is_leaderboard_configured() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { gravipop_leaderboard_configured() == 1 }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn leaderboard_refresh() {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { gravipop_leaderboard_refresh() };
    }
}

pub fn leaderboard_submit(name: &str, score: u64) {
    #[cfg(target_arch = "wasm32")]
    {
        let name_bytes = name.as_bytes();
        let high = (score >> 32) as u32;
        let low = (score & 0xffff_ffff) as u32;
        unsafe {
            gravipop_leaderboard_submit(name_bytes.as_ptr(), name_bytes.len() as u32, high, low);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, score);
    }
}

pub fn leaderboard_status() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let len = unsafe { gravipop_leaderboard_status_len() };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; len as usize];
        let copied = unsafe { gravipop_leaderboard_status_get(buf.as_mut_ptr(), len as u32) };
        if copied > 0 {
            buf.truncate(copied as usize);
            String::from_utf8(buf).unwrap_or_default()
        } else {
            String::new()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}

pub fn leaderboard_entries_json() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let len = unsafe { gravipop_leaderboard_data_len() };
        if len <= 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        let copied = unsafe { gravipop_leaderboard_data_get(buf.as_mut_ptr(), len as u32) };
        if copied > 0 {
            buf.truncate(copied as usize);
            String::from_utf8(buf).ok()
        } else {
            None
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
