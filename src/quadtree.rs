use image::GenericImage;
use image::{ImageBuffer, Rgb, RgbImage};
use itertools::Itertools;

pub(crate) struct QuadTree<'a> {
    img: &'a RgbImage,
    root: QuadTreeNode<'a>,
}

enum TreeDivision<'a> {
    Leaf,
    SplitX(QuadTreeNode<'a>, QuadTreeNode<'a>),
    SplitY(QuadTreeNode<'a>, QuadTreeNode<'a>),
}

struct QuadTreeNode<'a> {
    img: &'a RgbImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    subtree: Box<TreeDivision<'a>>,
}

impl<'a> QuadTreeNode<'a> {
    pub fn new(
        img: &'a RgbImage,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        tree_depth: u32,
    ) -> Self {
        let mut qt: QuadTreeNode<'a> = QuadTreeNode {
            img,
            x,
            y,
            width,
            height,
            subtree: Box::new(TreeDivision::Leaf),
        };
        if tree_depth > 0 {
            let ctr = width / 2;
            println!("subdivide image, tree_depth = {}", tree_depth);
            qt.subtree = Box::new(TreeDivision::SplitX(
                QuadTreeNode::new(&img, (x, y), (ctr, height), tree_depth - 1),
                QuadTreeNode::new(&img, (x + ctr, y), (width - ctr, height), tree_depth - 1),
            ));
        };
        qt
    }

    fn pixels(&self) -> impl Iterator<Item = &Rgb<u8>> {
        (self.x..(self.x + self.width))
            .cartesian_product(self.y..(self.y + self.height))
            .map(|(x, y)| self.img.get_pixel(x, y))
    }

    fn avg_pixel(&self) -> Rgb<u8> {
        let mut total_r: u32 = 0;
        let mut total_g: u32 = 0;
        let mut total_b: u32 = 0;
        let num_pixels: u32 = self.width * self.height;
        let mut count_num_pixels: u32 = 0;
        self.pixels().for_each(|pixel| {
            let Rgb([r, g, b]) = pixel;
            let r: u32 = (*r).into();
            let g: u32 = (*g).into();
            let b: u32 = (*b).into();
            total_r += r;
            total_g += g;
            total_b += b;
            count_num_pixels += 1;
        });
        // println!(
        //     "average looks like {:?}, num_pixels = {:?} or {}",
        //     (total_r, total_g, total_b),
        //     num_pixels,
        //     count_num_pixels
        // );
        Rgb([
            (total_r / num_pixels).try_into().unwrap(),
            (total_g / num_pixels).try_into().unwrap(),
            (total_b / num_pixels).try_into().unwrap(),
        ])
        // for (x, y) in (self.x..{
        //     let ref this = self;
        //     this.x + this.width
        // })
        //     .zip(self.y..(self.y + self.height))
        // {
        //     let pixel = self.img.get_pixel(self.x + x, self.y + y);
        // }
    }

    fn render(&self, out_image: &mut RgbImage) {
        match &*self.subtree {
            TreeDivision::Leaf => {
                let avg_pixel = self.avg_pixel();
                // println!("avg pixel content for this image = {:?}", avg_pixel);
                let rectangle: ImageBuffer<Rgb<u8>, Vec<u8>> =
                    ImageBuffer::from_fn(self.width, self.height, |_, _| -> Rgb<u8> {
                        // *self.img.get_pixel(x + self.x, y + self.y)
                        avg_pixel
                    });
                out_image.copy_from(&rectangle, self.x, self.y).unwrap();
            }
            TreeDivision::SplitX(left, right) => {
                left.render(out_image);
                right.render(out_image);
            }
            TreeDivision::SplitY(top, bottom) => {
                top.render(out_image);
                bottom.render(out_image);
            }
        }
    }
}

impl<'a> QuadTree<'a> {
    pub fn new(img: &'a RgbImage, tree_depth: u32) -> Self {
        let height = img.height();
        let width = img.width();
        let qt = QuadTree {
            img,
            root: QuadTreeNode::new(&img, (0, 0), (width, height), tree_depth),
        };
        qt
    }

    pub fn render(&self, out_path: String) {
        let mut out_image: RgbImage = RgbImage::new(self.img.width(), self.img.height());
        self.root.render(&mut out_image);
        out_image.save(out_path).expect("image rendering failed");
    }
}
