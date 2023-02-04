use engine::AppConfig;
use playground::MyGame;

fn main() {
    let game_config = AppConfig {
        name: "Ysera Playground",
        window_size: [1080, 720],
    };
    engine::create::<MyGame>(game_config);
}
