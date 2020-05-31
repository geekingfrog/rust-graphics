extern crate image;

fn main() {
    let imgx = 1 << 12;
    let imgy = 1 << 12;
    // let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
    let mut img = image::RgbImage::new(imgx, imgy);

    // // let (mut r, mut g, mut b) = (0,0,0);

    let mut r: i16 = 0;
    let mut g: i16 = 0;
    let mut b: i16 = 0;
    let mut rv = 1;
    let mut gv = 1;
    let mut bv = 1;
    let mut ra = 0;
    let mut ga = 0;
    let mut ba = 0;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // println!("({},{}) - {}, {}, {}", x, y, r, g, b);
        *pixel = image::Rgb([r as u8, g as u8, b as u8]);

        if (r == 0 && rv < 0) || (r == 255 && rv > 0) {
            ra = -rv;
            rv = 0;
        }

        if rv == 0 {
            if (g == 0 && gv < 0) || (g == 255 && gv > 0) {
                ga = -gv;
                gv = 0;
            }

            if gv == 0 {
                if (b == 0 && bv < 0) || (b == 255 && bv > 0) {
                    ba = -bv;
                    bv = 0;
                }

                b += bv;
                bv += ba;
                ba = 0;
            }

            g += gv;
            gv += ga;
            ga = 0;
        }

        r += rv;
        rv += ra;
        ra = 0;

    }

    img.save("outputs/coucou.jpg").unwrap()
}
