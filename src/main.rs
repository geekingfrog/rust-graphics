extern crate image;
extern crate num_traits;
extern crate rand;

use num_traits::int::PrimInt;
use rand::seq::SliceRandom;
use std::collections::{HashSet, BTreeSet, VecDeque};
use std::path::Path;

mod fractal;
mod sdl;

struct Canvas {
    max_x: u32,
    max_y: u32,
    img: image::RgbImage,
}

impl Canvas {
    fn new(imgx: u32, imgy: u32) -> Self {
        Canvas {
            max_x: imgx,
            max_y: imgy,
            img: image::RgbImage::new(imgx, imgy),
        }
    }

    // fn neighbours(self, x: u32, y: u32) -> impl Iterator<Item=(u32,u32)> {
    fn neighbours(&self, point: (u32, u32)) -> Vec<(u32, u32)> {
        let (x, y) = point;
        let mut result = Vec::new();
        if x > 0 {
            result.push((x - 1, y));
        }
        if x < self.max_x - 1 {
            result.push((x + 1, y));
        }

        if y > 0 {
            result.push((x, y - 1));
        }
        if y < self.max_y - 1 {
            result.push((x, y + 1));
        }

        result
    }

    fn put(&mut self, point: (u32, u32), col: [u8; 3]) {
        let (x, y) = point;
        self.img.put_pixel(x, y, image::Rgb(col));
    }

    fn save<Q: AsRef<Path>>(&self, path: Q) -> image::ImageResult<()> {
        self.img.save(path)
    }
}

fn main() {

    // fractal::test();
    sdl::test_sdl();
    return;

    let imgx = 1 << 8;
    let imgy = 1 << 8;

    let mut canvas = Canvas::new(imgx, imgy);

    // println!("{}, {:?}", imgx, canvas.neighbours(imgx - 1, 0));
    let mut rng = rand::thread_rng();
    let mut colors_tmp = gen_colors((imgx * imgy) as usize);

    // let mut canvas_ref = Canvas::new(imgx, imgy);
    // let mut i = 0;
    // for (_, _, pixel) in canvas_ref.img.enumerate_pixels_mut() {
    //     let c = colors_tmp[i];
    //     *pixel = image::Rgb([c.0, c.1, c.2]);
    //     i += 1;
    // }
    // canvas_ref.save("outputs/coucou_ref.jpg").unwrap();


    // colors_tmp.shuffle(&mut rng);
    let mut to_place = VecDeque::new();
    let mut placed = HashSet::new();
    let initial_point = (1<<7, 1<<7);
    let mut colors = colors_tmp.into_iter().map(|(r,g,b)| [r,g,b]).collect::<BTreeSet<_>>();

    let stuff = colors.iter().cloned().next();
    match stuff {
        None => panic!("must have at least one color"),
        Some(col) => {
            // let col = [*r, *g, *b];
            placed.insert(initial_point);
            canvas.put(initial_point, col);
            colors.remove(&col);
            for p in canvas.neighbours(initial_point) {
                to_place.push_back((col, p));
            }
        }
    }

    while let Some((from_col, point)) = to_place.pop_front() {
        if placed.contains(&point) {
            continue;
        }
        // let f = (from_col[0], from_col[1], from_col[2]);
        match colors
            .iter()
            .cloned()
            .min_by_key(|c| dist(&from_col, c))
        {
            None => panic!("no available color"),
            Some(col) => {
                canvas.put(point, col);
                placed.insert(point);
                colors.remove(&col);
                for n in canvas.neighbours(point) {
                    to_place.push_back((col, n));
                }
            }
        }
    }

    canvas.save("outputs/coucou.jpg").unwrap();

    // // let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
    // let mut img = image::RgbImage::new(imgx, imgy);
    //
    // let v = (0..=2)
    //     .flat_map(move |x| (0..=2).map(move |y| (x, y)))
    //     .collect::<Vec<_>>();
    // println!("{:?}", v);
    //
    // // // let (mut r, mut g, mut b) = (0,0,0);
    //
    // let mut r: i16 = 0;
    // let mut g: i16 = 0;
    // let mut b: i16 = 0;
    // let mut rv = 1;
    // let mut gv = 1;
    // let mut bv = 1;
    // let mut ra = 0;
    // let mut ga = 0;
    // let mut ba = 0;
    //
    // // let placed_pxs = HashSet::new();
    //
    // // println!("len: {}, product: {}", colors.len(), imgx * imgy);
    // // panic!("stop");
    //
    // let mut i = 0;
    // for (x, y, pixel) in img.enumerate_pixels_mut() {
    //     // println!("({},{}) - {}, {}, {}", x, y, r, g, b);
    //     let (r, g, b) = colors[i];
    //     i += 1;
    //     *pixel = image::Rgb([r as u8, g as u8, b as u8]);
    // }
    //
    // img.save("outputs/coucou.jpg").unwrap()
}

fn dist<T: PrimInt + std::fmt::Display>(a: &[T;3], b: &[T;3]) -> T {
// fn dist(a: &[u8;3], b: &[u8;3]) -> u32 {
    let (ax, ay, az) = (a[0], a[1], a[2]);
    let (bx, by, bz) = (b[0], b[1], b[2]);
    let dx = bx - ax;
    // println!("dx: {}", dx);
    let dy = by - ay;
    let dz = bz - az;
    (dx * dx) + (dy * dy) + (dz * dz)
}

/// generate a vector of `n` different colors. if `n < 2²⁴` it tries to
/// scale to use the full range [0;255] for each r,g,b
fn gen_colors(n: usize) -> Vec<(u8, u8, u8)> {
    let up = std::cmp::min((n as f32).cbrt().ceil() as usize, 255);
    let ratio = 255.0 / up as f32;

    let mut result = Vec::with_capacity(n);
    for b in 1..=up {
        for g in 1..=up {
            for r in 1..=up {
                result.push((
                    (r as f32 * ratio) as u8,
                    (g as f32 * ratio) as u8,
                    (b as f32 * ratio) as u8,
                ));
                if result.len() >= n {
                    return result;
                }
            }
        }
    }
    return result;
}
