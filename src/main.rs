// simple pong like game with 3 lives
// using minifb to render game space
// by maths.earth

extern crate minifb;
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

// Constants for window dimensions and frame timing
const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;
const FRAME_TARGET_TIME: u64 = 16; // ~60 FPS
const PAUSE_DURATION: Duration = Duration::from_secs(2);

struct GameObject {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    vel_x: f32,
    vel_y: f32,
}

struct Game {
    window: Window,
    ball: GameObject,
    paddle: GameObject,
    last_frame_time: Instant,
    game_is_running: bool,
    lives: i32,
    score: i32,
    is_paused: bool,
    pause_start: Option<Instant>,
    ball_reset_pending: bool,
}

impl Game {
    fn new() -> Self {
        let window = Window::new(
            "Game Window",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("Error creating window: {}", e);
        });

        let ball = GameObject {
            x: 20.0,
            y: 20.0,
            width: 15.0,
            height: 15.0,
            vel_x: 300.0,
            vel_y: 300.0,
        };

        let paddle = GameObject {
            width: 100.0,
            height: 20.0,
            x: (WINDOW_WIDTH as f32 / 2.0) - 50.0,
            y: WINDOW_HEIGHT as f32 - 40.0,
            vel_x: 0.0,
            vel_y: 0.0,
        };

        Game {
            window,
            ball,
            paddle,
            last_frame_time: Instant::now(),
            game_is_running: true,
            lives: 3,
            score: 0,
            is_paused: false,
            pause_start: None,
            ball_reset_pending: false,
        }
    }

    fn process_input(&mut self) {
        // Handle input for exiting the game
        if self.window.is_key_down(Key::Escape) {
            self.game_is_running = false;
        }

        // Handle paddle movement input
        if !self.is_paused {
            if self.window.is_key_down(Key::Left) {
                self.paddle.vel_x = -400.0;
            } else if self.window.is_key_down(Key::Right) {
                self.paddle.vel_x = 400.0;
            } else {
                self.paddle.vel_x = 0.0;
            }
        }
    }

    fn update(&mut self) {
        // Handle pause state
        if self.is_paused {
            if let Some(start) = self.pause_start {
                if start.elapsed() >= PAUSE_DURATION {
                    self.is_paused = false;
                    self.pause_start = None;
                    self.ball_reset_pending = true;
                    self.reset_ball();
                } else {
                    return;
                }
            }
        }

        // Ensure ball reset is handled before updating positions
        if self.ball_reset_pending {
            self.ball_reset_pending = false;
            self.last_frame_time = Instant::now(); // Reset the frame time to avoid large delta time
            return;
        }

        // Calculate delta time for consistent movement
        let current_time = Instant::now();
        let delta_time = (current_time - self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;

        // Update ball and paddle positions
        self.ball.x += self.ball.vel_x * delta_time;
        self.ball.y += self.ball.vel_y * delta_time;
        self.paddle.x += self.paddle.vel_x * delta_time;

        // Handle ball collision with window boundaries
        if self.ball.x <= 0.0 || self.ball.x + self.ball.width >= WINDOW_WIDTH as f32 {
            self.ball.vel_x = -self.ball.vel_x;
        }

        if self.ball.y <= 0.0 {
            self.ball.vel_y = -self.ball.vel_y;
        }

        // Handle ball collision with paddle
        if self.ball.y + self.ball.height >= self.paddle.y
            && self.ball.x + self.ball.width >= self.paddle.x
            && self.ball.x <= self.paddle.x + self.paddle.width
        {
            self.ball.vel_y = -self.ball.vel_y;
            self.score += 1;
        }

        // Prevent paddle from moving out of window boundaries
        if self.paddle.x <= 0.0 {
            self.paddle.x = 0.0;
        }

        if self.paddle.x >= WINDOW_WIDTH as f32 - self.paddle.width {
            self.paddle.x = WINDOW_WIDTH as f32 - self.paddle.width;
        }

        // Handle ball falling out of window (losing a life)
        if self.ball.y + self.ball.height > WINDOW_HEIGHT as f32 {
            self.lives -= 1;
            if self.lives > 0 {
                self.is_paused = true;
                self.pause_start = Some(Instant::now());
                // Move ball to a safe position off-screen before pausing
                self.ball.x = WINDOW_WIDTH as f32 / 2.0 - self.ball.width / 2.0;
                self.ball.y = WINDOW_HEIGHT as f32 / 2.0 - self.ball.height / 2.0;
                self.ball.vel_x = 0.0;
                self.ball.vel_y = 0.0;
            } else {
                self.game_is_running = false;
            }
        }
    }

    fn reset_ball(&mut self) {
        // Reset ball position and velocity
        self.ball.x = WINDOW_WIDTH as f32 / 2.0 - self.ball.width / 2.0;
        self.ball.y = WINDOW_HEIGHT as f32 / 2.0 - self.ball.height / 2.0;
        self.ball.vel_x = 300.0;
        self.ball.vel_y = 300.0;
    }

    fn render(&mut self, buffer: &mut [u32]) {
        // Clear the screen
        for i in buffer.iter_mut() {
            *i = 0;
        }

        // Render ball
        for y in 0..self.ball.height as usize {
            for x in 0..self.ball.width as usize {
                let index = (self.ball.y as usize + y) * WINDOW_WIDTH + (self.ball.x as usize + x);
                if index < buffer.len() {
                    buffer[index] = 0xFFFFFFFF;
                }
            }
        }

        // Render paddle
        for y in 0..self.paddle.height as usize {
            for x in 0..self.paddle.width as usize {
                let index = (self.paddle.y as usize + y) * WINDOW_WIDTH + (self.paddle.x as usize + x);
                if index < buffer.len() {
                    buffer[index] = 0xFFFFFFFF;
                }
            }
        }

        // Update window with buffer
        self.window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
    }
}

fn main() {
    let mut game = Game::new();
    let mut buffer: Vec<u32> = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT];

    // Main game loop
    while game.game_is_running && game.window.is_open() {
        game.process_input();
        game.update();
        game.render(&mut buffer);
        std::thread::sleep(Duration::from_millis(FRAME_TARGET_TIME));
    }

    println!("Game Over! Lives remaining: {}", game.lives);
    println!("Final Score: {}", game.score);
}
