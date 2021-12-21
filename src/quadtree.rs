use image::GenericImage;
use image::{ImageBuffer, Rgb, RgbImage};
use itertools::Itertools;

use crate::util;

pub(crate) struct QuadTree<'a> {
    img: &'a RgbImage,
    root: QuadTreeNode<'a>,
}
enum SubTree<'a> {
    Leaf,
    Split {
        nw: QuadTreeNode<'a>,
        ne: QuadTreeNode<'a>,
        sw: QuadTreeNode<'a>,
        se: QuadTreeNode<'a>,
    },
}

impl<'a> SubTree<'a> {
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

    fn get_children(&self) -> Vec<&QuadTreeNode<'a>> {
        match self {
            SubTree::Leaf => vec![],
            SubTree::Split { nw, ne, sw, se } => vec![nw, ne, sw, se],
        }
    }
}

// impl<'a> IntoIterator for SubTree<'a> {
//     type Item = &'a QuadTreeNode<'a>;

//     type IntoIter = std::vec::IntoIter<&'a QuadTreeNode<'a>>;

//     fn into_iter(self) -> Self::IntoIter {
//         match &self {
//             SubTree::Leaf => vec![].into_iter(),
//             SubTree::Split { nw, ne, sw, se } => vec![nw, ne, sw, se].into_iter(),
//         }
//     }
// }

struct QuadTreeNode<'a> {
    img: &'a RgbImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    subtree: Box<SubTree<'a>>,
    avg_pixel: Rgb<u8>,
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
            subtree: Box::new(SubTree::Leaf),
            // TODO use a lazy wrapper to avoid setting this default value before the calculation.
            avg_pixel: Rgb([0, 0, 0]),
        };
        // avoid division by zero
        qt.subdivide(tree_depth);
        qt.set_avg_pixel();
        qt
    }

    fn pixels(&self) -> impl Iterator<Item = &Rgb<u8>> {
        (self.x..(self.x + self.width))
            .cartesian_product(self.y..(self.y + self.height))
            .map(|(x, y)| self.img.get_pixel(x, y))
    }

    // recursively calculates the averagePixel for this square,
    // stores the result in self.avg_pixel, and returns it
    fn set_avg_pixel(&mut self) -> Rgb<u8> {
        match &mut *(self.subtree) {
            SubTree::Leaf => {
                self.avg_pixel = util::avg_pixels(self.pixels());
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
    fn subdivide(&mut self, tree_depth: u32) {
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
                            self.img,
                            (self.x, self.y),
                            (ctr_x, ctr_y),
                            tree_depth - 1,
                        ),
                        ne: QuadTreeNode::new(
                            self.img,
                            (self.x + ctr_x, self.y),
                            (self.width - ctr_x, ctr_y),
                            tree_depth - 1,
                        ),
                        sw: QuadTreeNode::new(
                            self.img,
                            (self.x, self.y + ctr_y),
                            (ctr_x, self.height - ctr_y),
                            tree_depth - 1,
                        ),
                        se: QuadTreeNode::new(
                            self.img,
                            (self.x + ctr_x, self.y + ctr_y),
                            (self.width - ctr_x, self.height - ctr_y),
                            tree_depth - 1,
                        ),
                    });
                }
            }
            SubTree::Split { .. } => {
                self.subtree.apply_mut(|q| {
                    q.subdivide(tree_depth - 1);
                });
                todo!("shouldn't use subdivide on non-leaf");
            }
        }
        self.subtree.apply_mut(|q| {
            q.set_avg_pixel();
        });
        // set the avg pixel for this node.
        self.set_avg_pixel();
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

    pub fn prune(&mut self, tolerance: u32) {
        let variance = util::calc_variance(self.pixels(), self.avg_pixel);
        match &mut *self.subtree {
            SubTree::Leaf => (),
            SubTree::Split { .. } => {
                if variance <= tolerance {
                    *self.subtree = SubTree::Leaf;
                } else {
                    self.subtree.apply_mut(|q| {
                        q.prune(tolerance);
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

impl<'a> QuadTree<'a> {
    pub fn new(img: &'a RgbImage, tree_depth: u32) -> Result<Self, String> {
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
        Ok(QuadTree { img, root })
    }

    // can replace with associated function
    pub fn render(&self, out_path: &str) {
        let mut out_image: RgbImage = RgbImage::new(self.img.width(), self.img.height());
        self.root.render(&mut out_image);
        out_image.save(out_path).expect("image rendering failed");
    }

    pub fn prune(&mut self, tolerance: u32) {
        self.root.prune(tolerance);
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
        let qt = QuadTree::new(&img, 1).unwrap();
        qt.root
            .apply(|q| assert_ne!(q.avg_pixel, qt.root.avg_pixel));
    }
}
