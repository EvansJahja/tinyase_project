use core::convert::Infallible;

use embedded_graphics::{Pixel, framebuffer::{Framebuffer, buffer_size, buffer_size_bpp}, pixelcolor::{BinaryColor, PixelColor, raw::{BigEndian, RawU1, RawU2}}, prelude::Point, primitives::Rectangle};
use embedded_graphics::prelude::*;

use crate::parser::chunk::embedded_graphics_impl::AlphaBinaryColor;


pub trait MyAwesomeTrait {
    const WIDTH: usize = 32;
    const HEIGHT: usize = 32;
    fn set_pixel2(&mut self, p: Point, c: AlphaBinaryColor);

    fn draw_iter2<I>(&mut self, pixels: I) -> Result<(), Infallible>
    where
        I: IntoIterator<Item = Pixel<AlphaBinaryColor>>;

    fn fill_contiguous2<I>(&mut self, area: Rectangle, colors: I) -> Result<(), Infallible> 
    where
        I: IntoIterator<Item = AlphaBinaryColor>;
}

impl MyAwesomeTrait for Framebuffer<BinaryColor, RawU1, BigEndian, 32, 32, {buffer_size_bpp(32, 32, RawU1::BITS_PER_PIXEL)}> 
{
    fn set_pixel2(&mut self, p: Point, c: AlphaBinaryColor) {
        let cc: RawU2 = c.into();
        match cc.into_inner() {
            1 => self.set_pixel(p, BinaryColor::On),
            2 => self.set_pixel(p, BinaryColor::Off),
            _ => {},
        };
    }

    fn draw_iter2<I>(&mut self, pixels: I) -> Result<(), Infallible>
        where
            I: IntoIterator<Item = Pixel<AlphaBinaryColor>> {
        for Pixel(p, c) in pixels {
            self.set_pixel2(p, c);
        }
        Ok(())
    }

    fn fill_contiguous2<I>(&mut self, area: Rectangle, colors: I) -> Result<(), Infallible> 
    where
        I: IntoIterator<Item = AlphaBinaryColor>,
    {
        self.draw_iter2(
            area.points()
                .zip(colors)
                .map(|(pos, color)| Pixel(pos, color)),
        )
    }

}