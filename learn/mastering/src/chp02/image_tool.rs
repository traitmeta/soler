use std::env;
use std::path::Path;

pub fn deal_images() {
    let image_path = env::args().nth(1).unwrap();
    let path = Path::new(&image_path);
    let img = image::open(path).unwrap();
    let rotated = img.rotate90();
    rotated.save(path).unwrap();
}
