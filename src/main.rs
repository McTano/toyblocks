use image::RgbImage;
use quadtree::QuadTree;

mod quadtree;
mod util;

fn main() {
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
            match QuadTree::new(&rgb_image, 5) {
                Ok(qt) => {
                    qt.render(&out_path);
                    pruning_test(&rgb_image, file_name);
                }
                Err(msg) => eprintln!("encountered error while creating quadtree: {}", msg),
            }
        } else {
            println!("encountered error while reading file list");
        }
    }
}

fn pruning_test<'a>(rgb_image: &'a RgbImage, file_name: String) -> QuadTree<'a> {
    println!("Now, let's try pruning to a reasonable variance");
    let mut qt2 = QuadTree::new(&rgb_image, 10).expect(
        "
error constructing wuadtree for pruning test",
    );
    for tolerance in [5, 10, 15] {
        qt2.prune(tolerance);
        qt2.render(&format!(
            "output/out-pruned-tolerance={}-{}",
            tolerance, file_name
        ));
    }
    qt2
}
