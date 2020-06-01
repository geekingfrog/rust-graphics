use image::{Rgb, RgbImage};
use num_complex::{Complex, Complex32};
use rayon::prelude::*;

pub fn test() {
    let factor = 1;
    let (img_x, img_y) = (1920 * factor, 1080 * factor);
    let (vx_min, vx_max): (f32, f32) = (-2.2, 1.0);

    let v_x = (vx_max - vx_min).abs();
    let v_y = v_x * img_y as f32 / img_x as f32;
    let (vy_min, _vy_max) = (-v_y / 2.0, v_y / 2.0);

    let dx = v_x / img_x as f32;
    let dy = v_y / img_y as f32;

    println!("{}", v_y);
    println!("{}", zn(50, Complex32::new(0.0, 0.0)));

    let mut img = RgbImage::new(img_x, img_y);
    let n_max = 100;

    let pixels = (0..img_x)
        .into_par_iter()
        .flat_map(move |x| {
            (0..img_y).into_par_iter().map(move |y| {
                let c_x = vx_min + x as f32 * dx;
                let c_y = vy_min + y as f32 * dy;
                let c = Complex32::new(c_x, c_y);
                let n = zn(n_max, c);

                if n == n_max {
                    return (x, y, Rgb([0,0,0]));
                }

                let red = 0;
                let green = n as f32 * 255.0 * 0.6 / n_max as f32;
                let blue = n * 255 / n_max;
                (x, y, Rgb([red, green as u8, blue as u8]))
            })
        })
        .collect::<Vec<_>>();

    for (x, y, col) in pixels {
        img.put_pixel(x, y, col);
    }

    // for x in 0..img_x {
    //     for y in 0..img_y {
    //         let c_x = vx_min + x as f32 * dx;
    //         let c_y = vy_min + y as f32 * dy;
    //         let c = Complex32::new(c_x, c_y);
    //         let n = zn(n_max, c);
    //         let col = (n_max - n) * 255 / n_max;
    //         img.put_pixel(x, y, Rgb([col as u8, 0, 0]));
    //     }
    // }

    img.save("outputs/fractal.jpg").unwrap();
}

fn zn(limit: u16, c: Complex<f32>) -> u16 {
    let mut z = Complex32::new(0.0, 0.0);
    for i in 0..limit {
        if z.norm() >= 2.0 {
            return i;
        }
        z = z * z + c;
    }
    limit
}
