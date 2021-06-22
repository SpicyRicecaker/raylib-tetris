use raylib::prelude::*;
use tetris_raylib_rs::config::Config;
use tetris_raylib_rs::Universe;

fn main() {
    let config = Config::default();
    let mut universe = Universe::default();

    init();

    let (mut rl, thread) = raylib::init()
        .size(*config.w() as i32, *config.h() as i32)
        .title(&config.title()[..])
        .build();

    rl.set_target_fps(*config.fps());

    // Debug, create new tetromino and add it to the universe

    while !rl.window_should_close() {
        universe.tick(&rl);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::from_hex("292828").unwrap());

        universe.render(&mut d, &config);
    }
}
