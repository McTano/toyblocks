use image::DynamicImage;
use quadtree::QuadTree;

mod quadtree;
mod util;

// fn crop_square(in_path: String, out_path: String) {
//     let img = image::open(in_path).unwrap();
//     if let DynamicImage::ImageRgb8(buf) = img {
//         let (width, height) = buf.dimensions();
//         let dim = std::cmp::min(width, height);
//         let square_out = ImageBuffer::from_fn(dim, dim, |x, y| *buf.get_pixel(x, y));
//         square_out.save(out_path).unwrap();
//     }
// }

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

            let img = image::open(in_path).unwrap();
            let rgb_image = match img {
                DynamicImage::ImageRgb8(rgb_image) => rgb_image,
                img => img.into_rgb8(),
            };
            let qt = QuadTree::new(&rgb_image, 10);

            qt.render(out_path);
        } else {
            println!("encountered error while reading file list");
        }
    }

    // for file_name in dogs {
    //     let in_path = format!("images/{}", file_name);
    //     let out_path = format!("output/tree-{}", file_name);

    //     let img = image::open(in_path).unwrap();
    //     let rgb_image = if let DynamicImage::ImageRgb8(rgb_image) = img {
    //         rgb_image
    //     } else {
    //         panic!("only Rgb8 images are supported for now.");
    //     };
    //     let qt = QuadTree::new(&rgb_image, 0);

    //     qt.render(out_path);
    //     // crop_square(in_path, out_path);

    //     // defaults to RGB8
    //     // println!(
    //     //     "inspect image: \n{:?}",
    //     //     match &img {
    //     //         ImageLuma8(_GrayImage) => "Luma8",
    //     //         ImageLumaA8(_GrayAlphaImage) => "LumaA8",
    //     //         ImageRgb8(_RgbImage) => "RGb8",
    //     //         ImageRgba8(_RgbaImage) => "Rgba8",
    //     //         ImageBgr8(_ImageBuffer) => "Bgr8",
    //     //         ImageBgra8(_ImageBuffer) => "Bgra8",
    //     //         ImageLuma16(_ImageBuffer) => "Luma16",
    //     //         ImageLumaA16(_ImageBuffer) => "LumaA16",
    //     //         ImageRgb16(_ImageBuffer) => "Rgb16",
    //     //         ImageRgba16(_ImageBuffer) => "Rgba16",
    //     //     }
    //     // );
    //     // let qt = QuadTree::new(img);
    //     // img.save(out_path)
    //     //     .expect("failed to write image to out file");
    // }
}
