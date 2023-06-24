use crate::paint::*;
use crate::prelude::*;
use crate::scene::*;
use crate::svg::*;
use crate::tesselate::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use std::rc::Rc;

/// General information on a scene
pub struct SceneCfg {
    pub theme: Chooser<ThemeItem>,
    pub distance: usize,
    pub deviation: usize,
    pub frame: Frame,
    pub pattern: Pattern,
    pub tiling: Tiling,
    pub nb_pattern: usize,
    pub var_stripes: usize,
    pub size_tiling: f64,
    pub nb_delaunay: usize,
    pub width_pattern: f64,
    pub line_width: f64,
    pub line_color: Color,
    pub tightness_spiral: f64,
}

/// A trait to box scene items and make them generic.
/// Spares us from a few lines of repeated code.
trait Dynamic<C>
where
    C: Contains + 'static,
{
    fn dynamic(self) -> Vec<Rc<dyn Contains>>;
}

impl<C> Dynamic<C> for Vec<C>
where
    C: Contains + 'static,
{
    fn dynamic(self) -> Vec<Rc<dyn Contains>> {
        self.into_iter()
            .map(|d| Rc::new(d) as Rc<dyn Contains>)
            .collect::<Vec<_>>()
    }
}

impl SceneCfg {
    /// Select a random color for a scene item.
    /// The actual color will depend on the Chooser<Color> with which it is mixed.
    pub fn choose_color(&self, rng: &mut StdRng) -> ColorItem {
        let ThemeItem(c, v, w, salt) = self
            .theme
            .choose(rng)
            .unwrap_or_else(|| ThemeItem(Color(0, 0, 0), None, None, Salt::none()));
        ColorItem {
            shade: Color::random(rng),
            deviation: v.unwrap_or(self.deviation),
            distance: w.unwrap_or(self.distance),
            theme: c,
            salt,
        }
    }

    /// Match pattern to function that generates it
    pub fn create_items(&self, rng: &mut StdRng) -> Vec<Rc<dyn Contains>> {
        match self.pattern {
            Pattern::FreeCircles => create_free_circles(rng, self).dynamic(),
            Pattern::FreeTriangles => create_free_triangles(rng, self).dynamic(),
            Pattern::FreeStripes => create_free_stripes(rng, self).dynamic(),
            Pattern::FreeSpirals => create_free_spirals(rng, self).dynamic(),
            Pattern::ConcentricCircles => create_concentric_circles(rng, self).dynamic(),
            Pattern::ParallelStripes => create_parallel_stripes(rng, self).dynamic(),
            Pattern::CrossedStripes => create_crossed_stripes(rng, self).dynamic(),
            Pattern::ParallelWaves => create_waves(rng, self).dynamic(),
            Pattern::ParallelSawteeth => create_sawteeth(rng, self).dynamic(),
        }
    }

    /// Math tiling to function that generates it
    pub fn make_tiling(&self, rng: &mut StdRng) -> Vec<(Pos, Path)> {
        match self.tiling {
            Tiling::Hexagons => tile_hexagons(&self.frame, self.size_tiling, rng.gen_range(0, 360)),
            Tiling::Triangles => {
                tile_triangles(&self.frame, self.size_tiling, rng.gen_range(0, 360))
            }
            Tiling::HexagonsAndTriangles => {
                tile_hybrid_hexagons_triangles(&self.frame, self.size_tiling, rng.gen_range(0, 360))
            }
            Tiling::SquaresAndTriangles => {
                tile_hybrid_squares_triangles(&self.frame, self.size_tiling, rng.gen_range(0, 360))
            }
            Tiling::Rhombus => tile_rhombus(
                &self.frame,
                self.size_tiling,
                (rng.gen::<f64>() * 0.6 + 0.4) * self.size_tiling,
                rng.gen_range(0, 360),
            ),
            Tiling::Delaunay => random_delaunay(&self.frame, rng, self.nb_delaunay),
            Tiling::Pentagons(n) => {
                let n = match n {
                    0 => rng.gen_range(1, 7),
                    n => n,
                };
                let ptiler = match n {
                    1 => pentagons_type1,
                    2 => pentagons_type2,
                    3 => pentagons_type3,
                    4 => pentagons_type4,
                    5 => pentagons_type5,
                    6 => pentagons_type6,
                    _ => unreachable!(),
                };
                ptiler(&self.frame, self.size_tiling, rng.gen_range(0, 360))
            }
        }
    }
}

/// Available patterns, open to additions
#[derive(Debug, Clone, Copy)]
pub enum Pattern {
    FreeCircles,
    FreeTriangles,
    FreeStripes,
    FreeSpirals,
    ConcentricCircles,
    ParallelStripes,
    CrossedStripes,
    ParallelWaves,
    ParallelSawteeth,
}

impl Pattern {
    /// Pick a random pattern (fallback if no other pattern choosing method is specified)
    pub fn choose(rng: &mut StdRng) -> Self {
        use Pattern::*;
        *vec![
            FreeCircles,
            FreeTriangles,
            FreeStripes,
            FreeSpirals,
            ConcentricCircles,
            ParallelStripes,
            CrossedStripes,
            ParallelWaves,
            ParallelSawteeth,
        ]
        .choose(rng)
        .unwrap()
    }
}

///Available tilings, open to additions
#[derive(Debug, Clone, Copy)]
pub enum Tiling {
    Hexagons,
    Triangles,
    HexagonsAndTriangles,
    SquaresAndTriangles,
    Rhombus,
    Delaunay,
    Pentagons(u8),
}

impl Tiling {
    /// Pick a random tiling (fallback if no other tiling choosing method is specified)
    pub fn choose(rng: &mut StdRng) -> Self {
        use Tiling::*;
        *vec![
            Hexagons,
            Triangles,
            HexagonsAndTriangles,
            SquaresAndTriangles,
            Rhombus,
            Delaunay,
            Pentagons(0),
        ]
        .choose(rng)
        .unwrap()
    }
}
