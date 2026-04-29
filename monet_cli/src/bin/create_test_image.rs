use image::{ImageBuffer, Rgba};

fn main() {
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(100, 100);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x < 50 && y < 50 {
            *pixel = Rgba([255, 0, 0, 255]);
        } else if x >= 50 && y < 50 {
            *pixel = Rgba([0, 255, 0, 255]);
        } else if x < 50 && y >= 50 {
            *pixel = Rgba([0, 0, 255, 255]);
        } else {
            *pixel = Rgba([255, 255, 0, 255]);
        }
    }
    img.save("test_wallpaper.png").unwrap();
    println!("Created test_wallpaper.png");
}
