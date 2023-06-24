use rand::{rngs::StdRng, Rng};
use std::convert::TryInto;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Color(pub usize, pub usize, pub usize);

impl Color {
    /// Ensure that all RGB values are within [[1; 100]]
    fn validate(mut self) -> Self {
        self.0 = self.0.min(255);
        self.1 = self.1.min(255);
        self.2 = self.2.min(255);
        self
    }

    /// Random noise
    pub fn variate(mut self, rng: &mut StdRng, amount: usize) -> Self {
        if amount > 0 {
            let amount = amount as isize;
            self.0 = (self.0 as isize + rng.gen_range(-amount, amount))
                .try_into()
                .unwrap_or(0);
            self.1 = (self.1 as isize + rng.gen_range(-amount, amount))
                .try_into()
                .unwrap_or(0);
            self.2 = (self.2 as isize + rng.gen_range(-amount, amount))
                .try_into()
                .unwrap_or(0);
        }
        self
    }

    /// Weighted mix with other color
    pub fn meanpoint(mut self, th: Self, distance: usize) -> Self {
        self.0 = (self.0 * distance + th.0 * (100 - distance)) / 100;
        self.1 = (self.1 * distance + th.1 * (100 - distance)) / 100;
        self.2 = (self.2 * distance + th.2 * (100 - distance)) / 100;
        self
    }

    /// Generate color
    pub fn random(rng: &mut StdRng) -> Self {
        Self(
            rng.gen_range(0, 255),
            rng.gen_range(0, 255),
            rng.gen_range(0, 255),
        )
    }
}

/// SVG color format: `rgb(<r>,<g>,<b>)`
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = self.validate();
        write!(f, "rgb({},{},{})", c.0, c.1, c.2)
    }
}
