use crate::epd5in83b_v2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BITS, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::*;
use crate::color::TriColor;
use crate::prelude::TriDisplay;

/// Full size buffer for use with the 5in83 EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2 * NUM_DISPLAY_BITS as usize]`
pub struct Display5in83 {
    buffer: [u8; 2 * NUM_DISPLAY_BITS as usize],
    rotation: DisplayRotation,
}

impl Display5in83 {
    fn draw_helper_tri_inverted(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<TriColor>,
    ) -> Result<(), Self::Error> {
        let rotation = self.rotation();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, bit) = find_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;
        let offset = self.chromatic_offset();

        let buffer = self.get_mut_buffer();

        // "Draw" the Pixel on that bit
        match color {
            TriColor::Black => {
                // clear bit in bw-buffer -> black
                buffer[index] &= !bit;
                // clear bit in chromatic buffer -> chromatic
                buffer[index + offset] &= !bit;
            }
            TriColor::White => {
                // set bit in bw-buffer -> white
                buffer[index] |= bit;
                // clear bit in chromatic buffer -> chromatic
                buffer[index + offset] &= !bit;
            }
            TriColor::Chromatic => {
                // clear bit in bw-buffer -> black
                buffer[index] &= !bit;
                // set bit in chromatic-buffer -> white
                buffer[index + offset] |= bit;
            }
        }
        Ok(())
    }
}

impl Default for Display5in83 {
    fn default() -> Self {
        Display5in83 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                2 * NUM_DISPLAY_BITS as usize],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display5in83 {
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper_tri_inverted(WIDTH, HEIGHT, pixel)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display5in83 {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl TriDisplay for Display5in83 {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    fn chromatic_offset(&self) -> usize {
        NUM_DISPLAY_BITS as usize
    }

    fn bw_buffer(&self) -> &[u8] {
        &self.buffer[0..self.chromatic_offset()]
    }

    fn chromatic_buffer(&self) -> &[u8] {
        &self.buffer[self.chromatic_offset()..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Black;
    use crate::color::Color;
    use crate::{epd5in83b_v2, epd7in5};
    use crate::graphics::{Display, DisplayRotation};
    use embedded_graphics::{
        prelude::*,
        primitives::{Line, PrimitiveStyle},
    };

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display5in83::default();
        assert_eq!(display.buffer().len(), 38400);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display5in83::default();
        for &byte in display.buffer() {
            assert_eq!(byte, epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display5in83::default();
        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = Display5in83::default();
        display.set_rotation(DisplayRotation::Rotate90);
        let _ = Line::new(Point::new(0, 632), Point::new(0, 639))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = Display5in83::default();
        display.set_rotation(DisplayRotation::Rotate180);
        let _ = Line::new(Point::new(632, 383), Point::new(639, 383))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display = Display5in83::default();
        display.set_rotation(DisplayRotation::Rotate270);
        let _ = Line::new(Point::new(383, 0), Point::new(383, 7))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
