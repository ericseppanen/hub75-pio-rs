use core::marker::PhantomData;
use embedded_graphics::prelude::*;
use libm::{powf, roundf};

pub trait Lut<const B: usize, C: RgbColor> {
    fn lookup(&self, color: C) -> (u16, u16, u16);
}

pub trait LutState {}
pub struct Uninit;
pub struct Init;
impl LutState for Uninit {}
impl LutState for Init {}

/// `POWB` must be `1 << B`
pub struct GammaLut<const POWB: usize, C: RgbColor, S>
{
    r: [u16; POWB],
    g: [u16; POWB],
    b: [u16; POWB],
    _color: PhantomData<C>,
    _state: PhantomData<S>,
}

impl<const POWB: usize, C: RgbColor> GammaLut<POWB, C, Uninit>
{
    pub const fn new() -> Self {
        Self {
            r: [0; POWB],
            g: [0; POWB],
            b: [0; POWB],
            _color: PhantomData,
            _state: PhantomData,
        }
    }

    pub fn init(mut self, gamma: (f32, f32, f32)) -> GammaLut<POWB, C, Init> {
        fn calculate_lookup_value(
            index: usize,
            source_max: u16,
            target_max: u16,
            gamma: f32,
        ) -> u16 {
            let max = target_max as f32;
            let remapped = index as f32 / source_max as f32 * max;
            let value = roundf(max * powf(remapped / max, gamma));
            u16::try_from(value as u32).unwrap_or(0)
        }

        let mut i = 0;
        while i < self.r.len() {
            self.r[i] = calculate_lookup_value(i, C::MAX_R as u16, POWB as u16 - 1, gamma.0);
            self.g[i] = calculate_lookup_value(i, C::MAX_G as u16, POWB as u16 - 1, gamma.1);
            self.b[i] = calculate_lookup_value(i, C::MAX_B as u16, POWB as u16 - 1, gamma.2);
            i += 1;
        }

        GammaLut {
            r: self.r,
            g: self.g,
            b: self.b,
            _color: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<const POWB: usize, C: RgbColor> Lut<POWB, C> for GammaLut<POWB, C, Init>
{
    fn lookup(&self, color: C) -> (u16, u16, u16) {
        let r = self.r[color.r() as usize];
        let g = self.g[color.g() as usize];
        let b = self.b[color.b() as usize];
        (r, g, b)
    }
}

pub struct Identity;

impl<const POWB: usize, C: RgbColor> Lut<POWB, C> for Identity
{
    fn lookup(&self, color: C) -> (u16, u16, u16) {
        (color.r() as u16, color.g() as u16, color.b() as u16)
    }
}
