pub mod color;
mod game;
mod input;
mod rotations;

use std::collections::HashSet;

use rotations::rotation_direction::RotationDirection;
use tetromino::tetromino_type::TetrominoType;

use color::ColorPalette;
use game::Game;

use super::*;
use direction::*;

const INITIAL_WIDTH: u32 = 10;
const INITIAL_HEIGHT: u32 = 20;

pub struct Universe {
    // Board
    dim: Dimensions,
    // Player controlled tetrimino
    focused_tetromino: Tetromino,
    ghost: Tetromino,
    // Tetriminos on board
    stagnant_tetrominos: Vec<Tetromino>,
    // Controls for tetrimino
    tetromino_controls: TetrominoControls,
    // Static color palette for game
    color_palette: ColorPalette,
    // Game mechanics
    game: Game,
}

pub struct Dimensions {
    w: u32,
    h: u32,
}

impl Dimensions {
    /// Get a reference to the dimensions's w.
    pub fn w(&self) -> &u32 {
        &self.w
    }

    /// Get a reference to the dimensions's h.
    pub fn h(&self) -> &u32 {
        &self.h
    }
}

impl Universe {
    pub fn new(
        dims: Dimensions,
        focused_tetromino: Tetromino,
        stagnant_tetrominos: Vec<Tetromino>,
        tetromino_controls: TetrominoControls,
        color_palette: ColorPalette,
        game: Game,
        ghost: Tetromino,
    ) -> Self {
        Universe {
            dim: dims,
            focused_tetromino,
            stagnant_tetrominos,
            tetromino_controls,
            color_palette,
            game,
            ghost,
        }
    }

    fn fall_focused(&mut self) {
        // Code that determines moving the pieces down
        let within_boundary = self
            .focused_tetromino
            .within_boundary(Tetromino::get_dxdy(Direction::Down), &self.dim);
        let mut collision = false;

        if within_boundary {
            collision = Tetromino::will_collide_all(
                &self.focused_tetromino,
                &self.stagnant_tetrominos,
                Tetromino::get_dxdy(Direction::Down),
            );
        }

        if !collision && within_boundary {
            self.focused_tetromino
                .move_by(Tetromino::get_dxdy(Direction::Down));
        } else {
            // Solidify the old current
            self.stagnant_tetrominos
                .push(self.focused_tetromino.clone());
            // Generate a new current
            self.focused_tetromino = TetrominoType::generate_tetromino_rand();

            // If it generates into a piece, game ova
            if Tetromino::will_collide_all(
                &self.focused_tetromino,
                &self.stagnant_tetrominos,
                [0, 0],
            ) {
                // Game over
                self.game.pause();
            }
        }
    }

    /// Implmentation of hard drop preview
    pub fn full_fall_focused(&mut self) {
        self.ghost = self.focused_tetromino.clone();
        loop {
            // Code that determines moving the pieces down
            let within_boundary = self
                .ghost
                .within_boundary(Tetromino::get_dxdy(Direction::Down), &self.dim);

            let mut collision = false;

            if within_boundary {
                collision = Tetromino::will_collide_all(
                    &self.ghost,
                    &self.stagnant_tetrominos,
                    Tetromino::get_dxdy(Direction::Down),
                );
            }

            if !collision && within_boundary {
                self.ghost
                    .move_by(Tetromino::get_dxdy(Direction::Down));
            } else {
                return;
            }
        }
    }

    fn clear(&mut self) {
        self.stagnant_tetrominos.clear();
    }

    fn game_over(&mut self, rl: &RaylibHandle) {
        if rl.is_key_down(KeyboardKey::KEY_R) {
            // Clear board
            self.clear();
            // Create new game instance
            self.game = Game::default();
        }
    }

    pub fn tick(&mut self, rl: &RaylibHandle) {
        if !self.game.running() {
            self.game_over(rl);
            return;
        }

        // Set level of the game

        self.game.tick();

        // update preview/ghost
        self.full_fall_focused();

        self.tetromino_controls.tick(rl);
        self.receive_key();

        // Literally just move current .y down
        // Falls at the rate of 6 per second

        if self.game.should_fall() {
            self.fall_focused();
        }

        let mut levels: HashMap<u32, u32> = HashMap::new();

        // Setup hash
        // We should probably store the hashmap, this way we won't have to update it every tick
        for tetromino in self.stagnant_tetrominos.iter() {
            for coord in tetromino.coords() {
                // Store the number of tetris parts in each y level
                let e = levels.entry(coord.y).or_insert(0);
                *e += 1;
            }
        }

        // filter out hash for levels that we need
        let levels = levels
            .iter()
            .filter_map(|l| if *l.1 == self.dim.w { Some(*l.0) } else { None })
            .collect::<HashSet<u32>>();

        // Nothing to do if there aren't any full rows
        if levels.is_empty() {
            return;
        }

        // ...Otherwise, if there is a full row...

        // Delete all stagnant tetriminos at these specific y levels
        let mut i = 0;
        while i != self.stagnant_tetrominos.len() {
            let mut j = 0;
            while j != self.stagnant_tetrominos[i].coords().len() {
                if levels.contains(&self.stagnant_tetrominos[i].coords()[j].y) {
                    self.stagnant_tetrominos[i].coords_mut().remove(j);
                } else {
                    j += 1;
                }
            }
            // No memory leaks thank you
            if self.stagnant_tetrominos[i].coords().is_empty() {
                self.stagnant_tetrominos.remove(i);
            } else {
                i += 1;
            }
        }

        // Then prepare to move the other tetriminos down (gravity)
        let mut diff = vec![0; self.dim.h as usize];
        for level in levels.iter() {
            Universe::change_arr_from_idx(&mut diff, *level, 1);
        }

        // Finally,if something happened try to move pieces down if they need to be moved
        // fk, we're iterating over stagnant tetrominos like 3 times. We honestly only need to really do it twice if we store the hashmap
        // If we implemented it with an array we would only need to iterate over the board once
        for i in 0..self.stagnant_tetrominos.len() {
            for j in 0..self.stagnant_tetrominos[i].coords().len() {
                self.stagnant_tetrominos[i].coords_mut()[j].y -=
                    diff[self.stagnant_tetrominos[i].coords()[j].y as usize];
            }
        }

        self.game.update(levels.len() as u32);
    }

    pub fn change_arr_from_idx(arr: &mut [u32], idx: u32, diff: u32) {
        for num in arr.iter_mut().skip(idx as usize) {
            *num += diff;
        }
    }

    /// Renders the 10x20 grid that tetrominos spawn on oo
    fn render_grid(&self, d: &mut RaylibDrawHandle, config: &Config) {
        // Spawn tetrminoes at up to level 22
        // Only show 10x20 grid

        let dx = *config.actual_w() as u32 / self.dim.w;
        // let dy = config.h() / self.h;

        for x in [0, self.dim.w].iter() {
            let current_x = x * dx + *config.canvas_l() as u32;
            d.draw_line_ex(
                Vector2 {
                    x: current_x as f32,
                    y: 0_f32,
                },
                Vector2 {
                    x: current_x as f32,
                    y: *config.h() as f32,
                },
                4_f32,
                self.color_palette.line(),
            );
        }

        // for x in (0..=self.w).into_iter() {
        //     // For every implement of x, draw from the ground to the ceiling
        //     let current_x = x * dx + *config.canvas_l() as u32;
        //     d.draw_line_ex(
        //         Vector2 {
        //             x: current_x as f32,
        //             y: 0_f32,
        //         },
        //         Vector2 {
        //             x: current_x as f32,
        //             y: *config.h() as f32,
        //         },
        //         0.5_f32,
        //         self.color_palette.line(),
        //     );
        // }
        // for y in (0..=self.h).into_iter() {
        //     let current_y = y * dy;
        //     d.draw_line_ex(
        //         Vector2 {
        //             x: *config.canvas_l() as f32,
        //             y: current_y as f32,
        //         },
        //         Vector2 {
        //             x: *config.canvas_r() as f32,
        //             y: current_y as f32,
        //         },
        //         0.5_f32,
        //         self.color_palette.line(),
        //     );
        // }
    }

    pub fn render(&self, d: &mut RaylibDrawHandle, config: &Config) {
        // Clear background
        d.clear_background(self.color_palette.grid());

        // Render grid
        self.render_grid(d, config);

        // Render the focused tetrimino
        self.focused_tetromino()
            .render(d, config, &self.dim, &self.color_palette);

        // And every other tetrimino
        for tetromino in self.stagnant_tetrominos().iter() {
            tetromino.render(d, config, &self.dim, &self.color_palette);
        }

        // Render the ghost
        self.ghost()
            .render_alpha(d, config, &self.dim, &self.color_palette);

        // If game is in an 'over' state
        if !self.game.running() {
            d.draw_text(
                "GAME",
                150,
                (*config.h() as f64 / 2_f64) as i32,
                100,
                self.color_palette.line(),
            );
            d.draw_text(
                "OVER",
                (*config.w() - 400) as i32,
                (*config.h() as f64 / 2_f64) as i32,
                100,
                self.color_palette.line(),
            );
            d.draw_text(
                "Press \"r\" to restart",
                150,
                (*config.h() as f64 / 2_f64) as i32 + 100,
                20,
                self.color_palette.line(),
            );
        } else {
            // Display level
            d.draw_text(
                &format!("LEVEL: {}", self.game.level()),
                150,
                150,
                50,
                self.color_palette.line(),
            );
            // Display score
            d.draw_text(
                &format!("score: {}", self.game.score()),
                150,
                200,
                30,
                self.color_palette.line(),
            )
        }
    }

}

// Getters and setters
impl Universe {
    /// Get a reference to the universe's current.
    pub fn focused_tetromino(&self) -> &Tetromino {
        &self.focused_tetromino
    }

    /// Get a mutable reference to the universe's current.
    pub fn focused_tetromino_mut(&mut self) -> &mut Tetromino {
        &mut self.focused_tetromino
    }

    /// Get a reference to the universe's ghost.
    pub fn ghost(&self) -> &Tetromino {
        &self.ghost
    }

    /// Get a reference to the universe's stagnant tetrominos.
    pub fn stagnant_tetrominos(&self) -> &Vec<Tetromino> {
        &self.stagnant_tetrominos
    }

    pub fn stagnant_tetrominos_mut(&mut self) -> &mut Vec<Tetromino> {
        &mut self.stagnant_tetrominos
    }
}

impl Default for Universe {
    fn default() -> Self {
        Universe::new(
            Dimensions {
                w: INITIAL_WIDTH,
                h: INITIAL_HEIGHT,
            },
            TetrominoType::generate_tetromino_rand(),
            vec![],
            TetrominoControls::default(),
            ColorPalette::default(),
            Game::default(),
            TetrominoType::generate_tetromino_rand(),
        )
    }
}
