extern crate sdl2;

use rand::Rng;
use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::VecDeque;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Side {
    Up,
    Right,
    Down,
    Left,
}

impl Into<usize> for Side {
    fn into(self) -> usize {
        match self {
            Side::Up => 1,
            Side::Right => 2,
            Side::Down => 3,
            Side::Left => 4,
        }
    }
}

#[derive(Clone, Copy)]
struct Tile<'texture> {
    texture: &'texture sdl2::render::Texture<'texture>,
    offset_x: i32,
    offset_y: i32,
    width: u32,
    height: u32,
}

impl<'a> fmt::Debug for Tile<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: *const sdl2::render::Texture<'a> = self.texture;
        f.debug_struct("Tile")
            .field("texture", &s)
            .field("offset_x", &self.offset_x)
            .field("offset_y", &self.offset_y)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl<'texture> PartialEq for Tile<'texture> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.texture, other.texture)
            && (self.offset_x == other.offset_x)
            && (self.offset_y == other.offset_y)
            && (self.width == other.width)
            && (self.height == other.height)
    }
}

impl<'texture> Eq for Tile<'texture> {}

impl<'texture> Hash for Tile<'texture> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.texture, state);
        self.offset_x.hash(state);
        self.offset_y.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

type Link<'a> = (&'a Tile<'a>, Side, &'a Tile<'a>);

// #[derive(Debug)]
// enum Cell<'a> {
//     Fixed(Tile<'a>),
//     Pending(&'a Vec<Tile<'a>>),
//     Impossible,
// }

type TileSupport = Vec<Vec<bool>>;

struct Grid<'a> {
    grid_x: usize,
    grid_y: usize,
    grid: Vec<Option<Tile<'a>>>,
    support: TileSupport,
}

impl<'a> Grid<'a> {
    fn new(x: usize, y: usize, tiles: &'a Vec<Tile<'a>>) -> Self {
        let grid = (0..(x * y)).into_iter().map(|_| None).collect();

        let support = (0..(x * y))
            .into_iter()
            .map(|_| {
                // all tiles possible at the beginning
                vec![true; tiles.len()]
            })
            .collect();

        Grid {
            grid_x: x,
            grid_y: y,
            grid,
            support,
        }
    }

    fn get(&self, (x, y): (usize, usize)) -> &Option<Tile<'a>> {
        &self.grid[self.grid_x * y + x]
    }

    fn get_support(&self, (x, y): (usize, usize)) -> &Vec<bool> {
        &self.support[self.grid_x * y + x]
    }

    fn set_support(&mut self, (x, y): (usize, usize), idx: usize, val: bool) {
        self.support[self.grid_x * y + x][idx] = val;
    }

    fn set(&mut self, (x, y): (usize, usize), cell: Tile<'a>) {
        self.grid[self.grid_x * y + x] = Some(cell);
    }

    /// returns a vector of (position, side), where the side is from the reference
    /// of the origin position.
    /// ((x-1,y), Left)  <- position -> ((x+1,y), Right)
    fn neighbours(&self, (x, y): (usize, usize)) -> Vec<((usize, usize), Side)> {
        let mut result = Vec::new();
        if x > 0 {
            result.push(((x - 1, y), Side::Left));
        }
        if x < self.grid_x - 1 {
            result.push(((x + 1, y), Side::Right));
        }
        if y > 0 {
            result.push(((x, y - 1), Side::Up));
        }
        if y < self.grid_y - 1 {
            result.push(((x, y + 1), Side::Down));
        }
        result
    }
}

pub fn test_sdl() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        // .resizable()
        .build()
        .unwrap();

    let _img_subsystem = image::init(image::InitFlag::all()).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let surf_tileset: sdl2::surface::Surface =
        image::LoadSurface::from_file("./resources/tileset2.png").unwrap();

    let texture_creator = canvas.texture_creator();
    let tileset = texture_creator
        .create_texture_from_surface(surf_tileset)
        .unwrap();

    let tiles = vec![
        Tile {
            texture: &tileset,
            offset_x: 0,
            offset_y: 0,
            width: 64,
            height: 64,
        },
        Tile {
            texture: &tileset,
            offset_x: 64 + 10,
            offset_y: 0,
            width: 64,
            height: 64,
        },
        Tile {
            texture: &tileset,
            offset_x: 154,
            offset_y: 0,
            width: 64,
            height: 64,
        },
        Tile {
            texture: &tileset,
            offset_x: 236,
            offset_y: 0,
            width: 64,
            height: 64,
        },
    ];

    let links = with_reverse(vec![
        // first tile with itself (minus symmetry)
        (&tiles[0], Side::Up, &tiles[0]),
        (&tiles[0], Side::Right, &tiles[0]),
        // first tile with the neighbourgs
        (&tiles[0], Side::Up, &tiles[2]),
        (&tiles[0], Side::Right, &tiles[1]),
        (&tiles[0], Side::Down, &tiles[2]),
        (&tiles[0], Side::Left, &tiles[1]),
        // second tile
        (&tiles[1], Side::Up, &tiles[1]),
        (&tiles[1], Side::Up, &tiles[3]),
        (&tiles[1], Side::Down, &tiles[1]),
        (&tiles[1], Side::Down, &tiles[3]),
        // third tile
        (&tiles[2], Side::Right, &tiles[2]),
        (&tiles[2], Side::Right, &tiles[3]),
        (&tiles[2], Side::Left, &tiles[2]),
        (&tiles[2], Side::Left, &tiles[3]),
        // fourth tile
        (&tiles[3], Side::Left, &tiles[3]),
    ]);

    let max_x: usize = 12;
    let max_y: usize = 8;

    let mut rng = rand::thread_rng();
    let mut grid = gen_grid(&mut rng, (max_x, max_y), &tiles, &links);
    render_grid(&mut canvas, &grid);

    'running: loop {
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    grid = gen_grid(&mut rng, (max_x, max_y), &tiles, &links);
                    render_grid(&mut canvas, &grid);
                }
                _ => {}
            }
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    // let mut canvas = window.into_canvas().build().unwrap();
    //
    // canvas.set_draw_color(Color::RGB(0, 255, 255));
    // canvas.clear();
    // canvas.present();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // let mut i = 0;
    // let start_time = std::time::Instant::now();
    // 'running: loop {
    //     i = (i + 1) % 255;
    //     canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
    //     canvas.clear();
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => break 'running,
    //             _ => {}
    //         }
    //     }
    //     // The rest of the game loop goes here...
    //
    //     if start_time.elapsed().as_secs() >= 1 {
    //         break 'running;
    //     }
    //
    //     canvas.present();
    //     std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    // }
    println!("exiting");
}

/// Given a list of link, return a list augmented with all the reverse
/// [(t1, side, t2)] -> [(t1, side, t2), (t2, opposite_side, t1)]
fn with_reverse<'a>(links: Vec<Link<'a>>) -> Vec<Link<'a>> {
    let mut result = Vec::new();
    for (t1, side, t2) in links {
        result.push((t1, side.clone(), t2));
        result.push((t2, opposite_side(side), t1));
    }
    result
}

fn opposite_side(side: Side) -> Side {
    match side {
        Side::Up => Side::Down,
        Side::Right => Side::Left,
        Side::Down => Side::Up,
        Side::Left => Side::Right,
    }
}

fn gen_grid<'a>(
    rng: &mut rand::rngs::ThreadRng,
    bounds: (usize, usize),
    tiles: &'a Vec<Tile<'a>>,
    links: &'a Vec<Link<'a>>,
) -> Grid<'a> {
    let (max_x, max_y) = bounds;
    let mut q = VecDeque::new();
    let mut grid;

    'gen: loop {
        q.clear();
        q.push_back((0, 0));
        grid = Grid::new(max_x, max_y, tiles);

        while let Some(coord) = q.pop_front() {
            let cell = grid.get(coord);
            match cell {
                Some(_) => (),
                None => {
                    let possible_tile_indexes = grid
                        .get_support(coord)
                        .iter()
                        .enumerate()
                        .filter(|(_, is_possible)| **is_possible)
                        .map(|(idx, _)| idx)
                        .collect::<Vec<_>>();

                    if possible_tile_indexes.is_empty() {
                        println!("reset grid");
                        continue 'gen;
                    }

                    let t_idx: usize = rng.gen::<usize>() % possible_tile_indexes.len();
                    let tile = tiles[possible_tile_indexes[t_idx]];
                    grid.set(coord, tile.clone());

                    for (next_coord, side) in grid.neighbours(coord) {
                        for (i, t) in tiles.iter().enumerate() {
                            let link = (&tile, side, t);
                            if !links.contains(&link) {
                                grid.set_support(next_coord, i, false);
                            }
                        }
                        q.push_back(next_coord);
                    }
                }
            }
        }
        break;
    }

    grid
}

fn render_grid<'a, T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    grid: &'a Grid<'a>,
) {
    for x in 0..grid.grid_x {
        for y in 0..grid.grid_y {
            match grid.get((x, y)) {
                Some(tile) => {
                    let dest = Rect::new(
                        x as i32 * tile.width as i32,
                        y as i32 * tile.height as i32,
                        tile.width,
                        tile.height,
                    );
                    let orig = Rect::new(tile.offset_x, tile.offset_y, tile.width, tile.height);
                    canvas.copy(tile.texture, orig, dest).unwrap();
                }
                _ => {}
            }
        }
    }
}
