use image::{imageops::colorops::ColorMap, Rgba};

#[derive(Clone, Copy)]
pub struct MyLevel;

impl ColorMap for MyLevel {
    type Color = Rgba<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Rgba<u8>) -> usize {
        let rgba = color.0;
        let brightness =
            ((rgba[0] as u32 * 299) + (rgba[1] as u32 * 587) + (rgba[2] as u32 * 114)) / 1000;
        if brightness > 195 { 1 } else { 0 }
    }

    #[inline(always)]
    fn lookup(&self, idx: usize) -> Option<Self::Color> {
        match idx {
            0 => Some([0, 0, 0, 1].into()),
            1 => Some([255, 255, 255, 1].into()),
            _ => None,
        }
    }

    /// Indicate NeuQuant implements `lookup`.
    fn has_lookup(&self) -> bool {
        true
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Rgba<u8>) {
        let new_color = 0xFF * self.index_of(color) as u8;
        let rgba = &mut color.0;
        rgba[0] = new_color;
    }
}
