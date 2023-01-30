use engine::winit::event::*;
use engine::{Engine, EurekaEngine};
use math::{vec3, Mat4, Vec3};
use std::collections::HashSet;
use std::ops::{Add, Mul};

#[derive(Default)]
pub struct MyGame {
    view: Mat4,
    camera_position: Vec3,
    camera_euler: Vec3,
    camera_view_dirty: bool,
    key_down_set_each_frame: HashSet<VirtualKeyCode>, // todo move to engine
}

impl MyGame {
    pub fn camera_yaw(&mut self, amount: f32) {
        self.camera_euler.y += amount;
        self.camera_view_dirty = true;
    }

    pub fn camera_pitch(&mut self, amount: f32) {
        self.camera_euler.x += amount;
        // avoid gimbal lock
        // let LIMIT = math::deg_to_rad(89.0);
        const LIMIT: f32 = 1.553_343; // 89 degrees, or equivalent to deg_to_rad(89.0f);
        self.camera_euler.x = f32::clamp(self.camera_euler.x, -LIMIT, LIMIT);

        self.camera_view_dirty = true;
    }

    pub fn camera_roll(&mut self, amount: f32) {
        self.camera_euler.z += amount;
        self.camera_view_dirty = true;
    }

    pub fn recalculate_view_matrix(&mut self) {
        if self.camera_view_dirty {
            let rotation = Mat4::from_euler_angles(
                self.camera_euler.z,
                self.camera_euler.x,
                self.camera_euler.y,
            );
            let translation = math::mat4_translation(self.camera_position);
            self.view = rotation.mul(translation);
            self.view.try_inverse_mut();
            self.camera_view_dirty = false;
            log::info!("app view {}", self.view);
        }
    }
}

impl EurekaEngine for MyGame {
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

        MyGame::default()
    }

    fn on_init(&mut self) {
        log::info!("MyGame on init");

        self.camera_position = vec3(0.0, 0.0, 3.0);
        let view = math::look_at(
            &vec3(2.0, 2.0, 2.0),
            &vec3(0.0, 0.0, 0.0),
            &vec3(0.0, 0.0, 1.0),
        );
        self.view = view;
        self.camera_euler = vec3(0.0, 0.0, 0.0);
        // self.view = math::mat4_translation(self.camera_position);
        // self.view.try_inverse_mut();
        self.camera_view_dirty = true;
    }

    fn on_update(&mut self, delta_time: f32, engine: &mut Engine) {
        // log::info!("MyGame on update dt: {}", delta_time);
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::A) {
            self.camera_yaw(100.0 * delta_time);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::D) {
            self.camera_yaw(-100.0 * delta_time);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::Up) {
            self.camera_pitch(100.0 * delta_time);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::Down) {
            self.camera_pitch(-100.0 * delta_time);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::Left) {
            self.camera_roll(100.0 * delta_time);
        }
        if self
            .key_down_set_each_frame
            .contains(&VirtualKeyCode::Right)
        {
            self.camera_roll(-100.0 * delta_time);
        }

        let temp_move_speed = 50f32;
        let mut velocity = vec3(0.0, 0.0, 0.0);

        if self.key_down_set_each_frame.contains(&VirtualKeyCode::W) {
            let forward = math::mat4_forward(self.view);
            velocity = velocity.add(forward);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::S) {
            let backward = math::mat4_backward(self.view);
            velocity = velocity.add(backward);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::Q) {
            let left = math::mat4_left(self.view);
            velocity = velocity.add(left);
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::E) {
            let right = math::mat4_right(self.view);
            velocity = velocity.add(right);
        }
        if self
            .key_down_set_each_frame
            .contains(&VirtualKeyCode::Space)
        {
            velocity.y += 1.0;
        }
        if self.key_down_set_each_frame.contains(&VirtualKeyCode::X) {
            velocity.y -= 1.0;
        }
        let z = vec3(0.0, 0.0, 0.0);
        if velocity != z {
            velocity.normalize_mut();
            self.camera_position.x += velocity.x * temp_move_speed * delta_time;
            self.camera_position.y += velocity.y * temp_move_speed * delta_time;
            self.camera_position.z += velocity.z * temp_move_speed * delta_time;
            self.camera_view_dirty = true;
        }
        self.recalculate_view_matrix();

        engine.renderer_set_view(self.view);

        self.key_down_set_each_frame.clear();
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
        // log::info!("MyGame on window input {:?}", keycode);
        self.key_down_set_each_frame.insert(keycode);
    }
}
