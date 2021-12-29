#![allow(dead_code)]
use image::{Pixel, Rgb};
use std::iter::Iterator;

// use std::iter::Step;

pub fn avg_pixels<'a>(pixels: impl IntoIterator<Item = &'a Rgb<u8>>) -> Rgb<u8> {
    let mut total_channels: [u32; 3] = [0, 0, 0];
    let mut num_pixels: u32 = 0;
    for pixel in pixels {
        for (total_x, x) in total_channels
            .iter_mut()
            .zip(pixel.channels().iter().map(|x| *x as u32))
        {
            *total_x += x;
        }
        num_pixels += 1;
    }
    Rgb(total_channels
        .map(|total_x| (total_x / (if num_pixels > 0 { num_pixels } else { 1 })) as u8))
}

pub fn calc_variance<'a>(
    pixels: impl IntoIterator<Item = &'a Rgb<u8>>,
    Rgb(avg_channels): Rgb<u8>,
) -> u32 {
    let mut variance: u64 = 0;
    let mut num_pixels = 0;
    let avg_channels: [i64; 3] = avg_channels.map(|x| x as i64);
    for Rgb(curr_channels) in pixels {
        for (avg, curr) in itertools::zip(avg_channels, curr_channels) {
            variance += (i64::abs(avg - *curr as i64) / 3) as u64;
        }
        num_pixels += 1
    }
    // println!("variance = {}", variance / num_pixels);
    return (variance / num_pixels) as u32;
}

// I could make this generic for different pixel types if I made the user pass in a &mut TPixel for the result.
// (maybe. I tried this and it was hard. Doing the casts is weird with a generic type);
// pub fn calc_avg_pixel<'a, T, U>(
//     out_pixel: &mut T,
//     pixels: impl Iterator<Item = &'a Rgb<u8>>,
// ) -> Rgb<u8>
// where
//     T: Subpixel = U>,
//     U: Primitive,
// {
//     let mut total_channels: std::slice::Iter<&u32> = out_pixel.channels().iter().map_into::<u32>();
//     for p in total_channels {
//         *p = 0;
//     }
//     // let mut total_channels: &mut [u32] = Array::out_pixel.channels().iter().map_into::<u32>();
//     for p in total_channels {
//         *p = 0 as U;
//     }
//     let mut num_pixels: u32 = 0;
//     for pixel in pixels {
//         for (total_x, x) in total_channels
//             .iter_mut()
//             .zip(pixel.channels().iter().map(|x| *x as u32))
//         {
//             *total_x += x;
//         }
//         num_pixels += 1;
//     }
//     Rgb(total_channels.map(|total_x| (total_x / num_pixels) as u8))
// }
