mod quadtree;
mod util;
use std::env;

use image::RgbImage;
use quadtree::QuadTree;

const DEFAULT_TREE_DEPTH: u32 = 20;

fn main() {
    let args: Vec<String> = env::args().collect();
    let in_path = &args[1];
    let out_path = &args[2];
    let tolerance: u32 = args[3]
        .parse()
        .expect("Expected tolerance would be valid unsigned integer");
    compress_image(in_path, tolerance, out_path);
}

fn compress_image(in_path: &String, tolerance: u32, out_path: &String) {
    let img: RgbImage = image::open(in_path)
        .expect("failed to open image")
        .into_rgb8();
    let mut qt = QuadTree::new(img.clone(), DEFAULT_TREE_DEPTH);
    qt.prune(tolerance);
    qt.render(out_path);
    println!("rendered compressed image at {}", out_path);
}

mod test {
    use image::RgbImage;

    use crate::quadtree::QuadTree;

    #[allow(dead_code)]
    fn pruning_test(rgb_image: RgbImage, file_name: String) -> QuadTree {
        println!("Now, let's try pruning to a reasonable variance");
        let mut qt2 = QuadTree::new(rgb_image, 10);
        for tolerance in [5, 10, 15] {
            qt2.prune(tolerance);
            qt2.render(&format!(
                "output/out-pruned-tolerance={}-{}",
                tolerance, file_name
            ));
        }
        qt2
    }

    #[test]
    #[ignore]
    fn big_test() {
        let files = std::fs::read_dir("images").expect("couldn't read images directory");
        for file in files {
            if let Ok(file) = file {
                let in_path = file.path();
                let file_name = file
                    .file_name()
                    .into_string()
                    .expect("failed to convert file_name to string");
                let out_path = format!("output/out-{}", file_name);
                let img = image::open(in_path).expect("failed to open img");
                let rgb_image: RgbImage = img.into_rgb8();
                let qt = QuadTree::new(rgb_image.clone(), 5);
                qt.render(&out_path);
                pruning_test(rgb_image, file_name);
            } else {
                println!("encountered error while reading file list");
            }
        }
    }
}
