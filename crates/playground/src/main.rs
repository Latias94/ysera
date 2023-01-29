use engine::winit::event::*;
use engine::GameConfig;

pub struct MyGame {}

impl engine::Game for MyGame {
    fn new() -> Self
    where
        Self: Sized,
    {
        std::env::set_var("RUST_BACKTRACE", "full");
        std::env::set_var("RUST_LOG", "debug");

        // profiling::tracy_client::Client::start();

        let mut builder = env_logger::Builder::from_default_env();
        builder.target(env_logger::Target::Stdout);
        builder.init();

        MyGame {}
    }

    fn on_init(&mut self) {
        log::info!("MyGame on init");
    }

    fn on_update(&mut self, _delta_time: f32) {
        // log::info!("MyGame on update dt: {}", delta_time);
    }

    fn on_render(&mut self, _delta_time: f32) {
        // log::info!("MyGame on render dt: {}", delta_time);
    }

    fn on_shutdown(&mut self) {
        log::info!("MyGame on Shutdown");
    }

    fn on_window_resize(&mut self, width: u32, height: u32) {
        log::info!("MyGame on window resize w:{} h:{}", width, height);
    }

    fn on_window_input(&mut self, keycode: VirtualKeyCode) {
        log::info!("MyGame on window input {:?}", keycode);
    }
}

fn main() {
    let game_config = GameConfig {
        name: "Eureka Playground",
        window_size: [1080, 720],
    };
    engine::create::<MyGame>(game_config);
}
