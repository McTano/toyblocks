#![allow(dead_code)]
use once_cell::sync::Lazy;

use image::GenericImage;
use image::{ImageBuffer, Rgb, RgbImage};
use itertools::Itertools;

use crate::util;

pub(crate) struct QuadTree {
    img: RgbImage,
    root: QuadTreeNode,
}

struct QuadTreeNode {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    subtree: Box<SubTree>,
    avg_pixel: Lazy<Rgb<u8>>,
}
enum SubTree {
    Leaf,
    Split {
        nw: QuadTreeNode,
        ne: QuadTreeNode,
        sw: QuadTreeNode,
        se: QuadTreeNode,
    },
}

impl SubTree {
    fn apply_mut(&mut self, mut func: impl FnMut(&mut QuadTreeNode) -> ()) {
        match self {
            SubTree::Leaf => (),
            SubTree::Split { nw, ne, sw, se } => {
                for q in [nw, ne, sw, se] {
                    func(q);
                }
            }
        }
    }

    fn apply(&self, func: impl Fn(&QuadTreeNode) -> ()) {
        if let SubTree::Split { nw, ne, sw, se } = self {
            for q in [nw, ne, sw, se] {
                func(q);
            }
        }
    }

    fn get_children(&self) -> Vec<&QuadTreeNode> {
        match self {
            SubTree::Leaf => vec![],
            SubTree::Split { nw, ne, sw, se } => vec![nw, ne, sw, se],
        }
    }
}

// impl IntoIterator for SubTree {
//     type Item = & QuadTreeNode;

//     type IntoIter = std::vec::IntoIter<& QuadTreeNode>;

//     fn into_iter(self) -> Self::IntoIter {
//         match &self {
//             SubTree::Leaf => vec![].into_iter(),
//             SubTree::Split { nw, ne, sw, se } => vec![nw, ne, sw, se].into_iter(),
//         }
//     }
// }

impl QuadTreeNode {
    pub fn new(
        img: &RgbImage,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        tree_depth: u32,
    ) -> Self {
        let mut qt: QuadTreeNode = QuadTreeNode {
            x,
            y,
            width,
            height,
            subtree: Box::new(SubTree::Leaf),
            // TODO use a lazy wrapper to avoid setting this default value before the calculation.
            avg_pixel: Rgb([0, 0, 0]),
        };
        // avoid division by zero
        qt.subdivide(tree_depth, &img);
        qt
    }

    fn pixels<'a>(&self, img: &'a RgbImage) -> impl Iterator<Item = &'a Rgb<u8>> {
        (self.x..(self.x + self.width))
            .cartesian_product(self.y..(self.y + self.height))
            .map(|(x, y)| img.get_pixel(x, y))
    }

    // recursively calculates the averagePixel for this square,
    // stores the result in self.avg_pixel, and returns it
    fn set_avg_pixel(&mut self, img: &RgbImage) -> Rgb<u8> {
        match &mut *(self.subtree) {
            SubTree::Leaf => {
                self.avg_pixel = util::avg_pixels(self.pixels(img));
                self.avg_pixel
            }
            SubTree::Split { nw, ne, sw, se } => {
                self.avg_pixel = util::avg_pixels(
                    [nw, ne, sw, se]
                        .iter()
                        .map(|quadrant| -> &Rgb<u8> { &quadrant.avg_pixel }),
                );
                self.avg_pixel
            }
        }
    }

    // divide the tree tree_depth times and then set
    // the average pixel on the current node
    fn subdivide(&mut self, tree_depth: u32, img: &RgbImage) {
        match &mut *self.subtree {
            SubTree::Leaf => {
                if self.width / 2 <= 0 || self.height / 2 <= 0 {
                    // eprintln!("subsquares got too small at level {}", tree_depth);
                } else if tree_depth > 0 {
                    // TODO: log a warning or something if the tree goes too small
                    let ctr_x = self.width / 2;
                    let ctr_y = self.height / 2;
                    // println!("subdivide image, tree_depth = {}", tree_depth);
                    self.subtree = Box::new(SubTree::Split {
                        nw: QuadTreeNode::new(
                            &img,
                            (self.x, self.y),
                            (ctr_x, ctr_y),
                            tree_depth - 1,
                        ),
                        ne: QuadTreeNode::new(
                            &img,
                            (self.x + ctr_x, self.y),
                            (self.width - ctr_x, ctr_y),
                            tree_depth - 1,
                        ),
                        sw: QuadTreeNode::new(
                            &img,
                            (self.x, self.y + ctr_y),
                            (ctr_x, self.height - ctr_y),
                            tree_depth - 1,
                        ),
                        se: QuadTreeNode::new(
                            &img,
                            (self.x + ctr_x, self.y + ctr_y),
                            (self.width - ctr_x, self.height - ctr_y),
                            tree_depth - 1,
                        ),
                    });
                }
            }
            SubTree::Split { .. } => {
                self.subtree.apply_mut(|q| {
                    q.subdivide(tree_depth - 1, img);
                });
                todo!("shouldn't use subdivide on non-leaf");
            }
        }
        self.subtree.apply_mut(|q| {
            q.set_avg_pixel(img);
        });
        // set the avg pixel for this node.
        self.set_avg_pixel(img);
    }

    pub fn apply_mut(&mut self, func: impl FnMut(&mut QuadTreeNode) -> ()) {
        self.subtree.apply_mut(func);
    }

    pub fn apply(&self, func: impl Fn(&QuadTreeNode) -> ()) {
        self.subtree.apply(func);
    }

    fn get_children(&self) -> Vec<&QuadTreeNode> {
        self.subtree.get_children()
    }

    fn render(&self, out_image: &mut RgbImage) {
        match &*self.subtree {
            SubTree::Leaf => {
                let avg_pixel = self.avg_pixel;
                // println!("avg pixel content for this image = {:?}", avg_pixel);
                let rectangle: ImageBuffer<Rgb<u8>, Vec<u8>> =
                    ImageBuffer::from_fn(self.width, self.height, |_, _| -> Rgb<u8> {
                        // *self.img.get_pixel(x + self.x, y + self.y)
                        avg_pixel
                    });
                out_image.copy_from(&rectangle, self.x, self.y).unwrap();
            }
            SubTree::Split { .. } => {
                for q in self.get_children() {
                    q.render(out_image);
                }
            }
        }
    }

    pub fn prune(&mut self, tolerance: u32, img: &RgbImage) {
        let variance = util::calc_variance(self.pixels(img), self.avg_pixel);
        match &mut *self.subtree {
            SubTree::Leaf => (),
            SubTree::Split { .. } => {
                if variance <= tolerance {
                    *self.subtree = SubTree::Leaf;
                } else {
                    self.subtree.apply_mut(|q| {
                        q.prune(tolerance, img);
                    })
                }
            }
        }
    }

    fn tree_height(&self) -> u32 {
        match &*self.subtree {
            SubTree::Leaf => 0,
            SubTree::Split { nw, ne, sw, se } => {
                [nw, ne, sw, se]
                    .iter()
                    .map(|qt| qt.tree_height())
                    .max()
                    .unwrap()
                    + 1
            }
        }
    }
}

impl QuadTree {
    pub fn new(img: RgbImage, tree_depth: u32) -> Self {
        let height = img.height();
        let width = img.width();
        if height == 0 || width == 0 {
            panic!("image is 0 pixels");
        }
        let root = QuadTreeNode::new(&img, (0, 0), (width, height), tree_depth);
        eprintln!(
            "requested height: {}, actual height = {}",
            tree_depth,
            root.tree_height()
        );
        QuadTree { img, root }
    }

    // can replace with associated function
    pub fn render(&self, out_path: &str) {
        let mut out_image: RgbImage = RgbImage::new(self.img.width(), self.img.height());
        self.root.render(&mut out_image);
        out_image.save(out_path).expect("image rendering failed");
    }

    pub fn prune(&mut self, tolerance: u32) {
        self.root.prune(tolerance, &self.img);
    }
}

#[cfg(test)]
mod test {
    use super::QuadTree;

    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }

    #[test]
    fn children_get_unique_pixels() {
        let img = "images/balloons_huey.jpg";
        let img = image::open(img).unwrap().to_rgb8();
        let qt = QuadTree::new(img.clone(), 1);
        qt.root
            .apply(|q| assert_ne!(q.avg_pixel, qt.root.avg_pixel));
    }
}
