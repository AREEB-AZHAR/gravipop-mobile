// Desktop entry point — delegates entirely to the library's game loop.
// Android uses android_main() in lib.rs instead.
use gravipop_mobile::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    gravipop_mobile::game_main().await;
}
