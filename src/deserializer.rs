use crate::cfg::SceneCfg;
use crate::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom};
use serde_derive::Deserialize;
use std::collections::HashMap;
use toml::{map::Map, Value};

const BASE_WEIGHT: usize = 10;

/// All config information
#[derive(Deserialize, Default, Debug)]
pub struct MetaConfig {
    pub global: Option<ConfigGlobal>,
    pub lines: Option<ConfigLines>,
    pub colors: Option<ConfigColors>,
    pub themes: Option<ConfigThemes>,
    pub shapes: Option<ConfigShapes>,
    pub data: Option<ConfigData>,
    pub entry: Option<Vec<ConfigEntry>>,
}

/// Global options
#[derive(Deserialize, Default, Debug)]
pub struct ConfigGlobal {
    pub deviation: Option<usize>,
    pub weight: Option<usize>, // Artifact of previous name
    pub distance: Option<usize>,
    pub size: Option<f64>,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

/// Lines appearance
#[derive(Deserialize, Default, Debug)]
pub struct ConfigLines {
    pub width: Option<f64>,
    pub color: Option<String>,
    pub del_width: Option<f64>,
    pub del_color: Option<String>,
    pub hex_width: Option<f64>,
    pub hex_color: Option<String>,
    pub tri_width: Option<f64>,
    pub tri_color: Option<String>,
    pub rho_width: Option<f64>,
    pub rho_color: Option<String>,
    pub hex_and_tri_width: Option<f64>,
    pub hex_and_tri_color: Option<String>,
    pub squ_and_tri_width: Option<f64>,
    pub squ_and_tri_color: Option<String>,
    pub pen_width: Option<f64>,
    pub pen_color: Option<String>,
}

/// Color list
#[derive(Deserialize, Default, Debug)]
pub struct ConfigColors {
    #[serde(flatten)]
    pub list: Map<String, Value>,
}

/// Theme list
#[derive(Deserialize, Default, Debug)]
pub struct ConfigThemes {
    #[serde(flatten)]
    pub list: Map<String, Value>,
}

/// Shapes combination list
#[derive(Deserialize, Default, Debug)]
pub struct ConfigShapes {
    #[serde(flatten)]
    pub list: Map<String, Value>,
}

/// Group together pattern options and tiling options
#[derive(Deserialize, Default, Debug)]
pub struct ConfigData {
    pub patterns: Option<ConfigPatterns>,
    pub tilings: Option<ConfigTilings>,
}

/// Tiling options
#[derive(Deserialize, Default, Debug)]
pub struct ConfigTilings {
    pub size_hex: Option<f64>,
    pub size_tri: Option<f64>,
    pub size_hex_and_tri: Option<f64>,
    pub size_squ_and_tri: Option<f64>,
    pub size_rho: Option<f64>,
    pub size_pen: Option<f64>,
    pub nb_delaunay: Option<usize>,
}

/// Pattern options
#[derive(Deserialize, Default, Debug)]
pub struct ConfigPatterns {
    pub nb_free_circles: Option<usize>,
    pub nb_free_spirals: Option<usize>,
    pub nb_free_stripes: Option<usize>,
    pub nb_crossed_stripes: Option<usize>,
    pub nb_parallel_stripes: Option<usize>,
    pub nb_concentric_circles: Option<usize>,
    pub nb_free_triangles: Option<usize>,
    pub nb_parallel_waves: Option<usize>,
    pub nb_parallel_sawteeth: Option<usize>,
    pub var_parallel_stripes: Option<usize>,
    pub var_crossed_stripes: Option<usize>,
    pub width_spiral: Option<f64>,
    pub width_stripe: Option<f64>,
    pub width_wave: Option<f64>,
    pub width_sawtooth: Option<f64>,
    pub tightness_spiral: Option<f64>,
}

/// Entry for a single theme/time combination
#[derive(Deserialize, Debug)]
pub struct ConfigEntry {
    pub span: Option<String>,
    pub distance: Option<usize>,
    pub themes: Option<Vec<String>>,
    pub shapes: Option<Vec<String>>,
    pub line_color: Option<String>,
}

impl MetaConfig {
    /// Parse from TOML.
    /// Heavy lifting done by external crates
    pub fn from_string(src: String) -> Self {
        toml::from_str(src.as_str()).unwrap_or_else(|_e| MetaConfig::default())
    }

    /// Choose options at random according to configuration
    pub fn pick_cfg(self, rng: &mut StdRng, time: u64) -> SceneCfg {
        // Read default/overriden global options
        let (deviation, distance, size, width, height) = {
            let (deviation, distance, size, width, height);
            match self.global {
                None => {
                    deviation = DEVIATION;
                    distance = DISTANCE;
                    size = SIZE;
                    width = WIDTH;
                    height = HEIGHT;
                }
                Some(g) => {
                    match g.deviation {
                        None => {
                            deviation = DEVIATION;
                        }
                        Some(d) => deviation = d,
                    }
                    match g.distance {
                        None => {
                            distance = g.weight.unwrap_or(DISTANCE);
                        }
                        Some(w) => distance = w,
                    }
                    match g.size {
                        None => {
                            size = SIZE;
                        }
                        Some(s) => {
                            size = s;
                        }
                    }
                    match g.width {
                        None => {
                            width = WIDTH;
                        }
                        Some(w) => {
                            width = w;
                        }
                    }
                    match g.height {
                        None => {
                            height = HEIGHT;
                        }
                        Some(s) => {
                            height = s;
                        }
                    }
                }
            }
            (deviation, distance, size, width, height)
        };

        // Get list of named colors
        let colors = {
            let mut colors = HashMap::new();
            if let Some(ConfigColors { list }) = self.colors {
                for name in list.keys() {
                    match color_from_value(&list[name], &colors) {
                        Ok(c) => {
                            colors.insert(name.clone(), c);
                        }
                        Err(_s) => {}
                    }
                }
            }
            colors
        };

        // Get list of named themes
        let mut themes = {
            let mut themes = HashMap::new();
            if let Some(ConfigThemes { list }) = self.themes {
                for name in list.keys() {
                    match theme_from_value(&list[name], &colors, &themes) {
                        Ok(th) => {
                            themes.insert(name.clone(), th);
                        }
                        Err(_s) => {}
                    }
                }
            }
            themes
        };

        // List of allowed shape combinations
        let shapes = {
            let mut shapes = HashMap::new();
            if let Some(ConfigShapes { list }) = self.shapes {
                for name in list.keys() {
                    shapes.insert(name.clone(), shapes_from_value(&list[name], &shapes));
                }
            }
            shapes
        };

        let (theme, shape, line_color_override) = choose_theme_shapes(rng, &self.entry, time);
        let (tiling, pattern) = match shapes.get(&shape) {
            None => (Tiling::choose(rng), Pattern::choose(rng)),
            Some(t) => (
                t.1.choose(rng).unwrap_or_else(|| Tiling::choose(rng)),
                t.0.choose(rng).unwrap_or_else(|| Pattern::choose(rng)),
            ),
        };

        // Get pattern-specific information according to picked shapes
        let (nb_pattern, var_stripes, width_pattern, tightness_spiral) = {
            let nb_pattern;
            let (mut var_stripes, mut width_pattern, mut tightness_spiral) = (0, 0.0, 0.0);
            if let Some(ConfigData {
                patterns: Some(p),
                tilings: _,
            }) = &self.data
            {
                match pattern {
                    Pattern::FreeCircles => {
                        nb_pattern = p.nb_free_circles.unwrap_or(NB_FREE_CIRCLES);
                    }
                    Pattern::FreeTriangles => {
                        nb_pattern = p.nb_free_triangles.unwrap_or(NB_FREE_TRIANGLES);
                    }
                    Pattern::FreeStripes => {
                        nb_pattern = p.nb_free_stripes.unwrap_or(NB_FREE_STRIPES);
                        width_pattern = p.width_stripe.unwrap_or(WIDTH_STRIPE);
                    }
                    Pattern::FreeSpirals => {
                        nb_pattern = p.nb_free_spirals.unwrap_or(NB_FREE_SPIRALS);
                        width_pattern = p.width_spiral.unwrap_or(WIDTH_SPIRAL);
                        tightness_spiral = p.tightness_spiral.unwrap_or(TIGHTNESS_SPIRAL);
                    }
                    Pattern::ConcentricCircles => {
                        nb_pattern = p.nb_concentric_circles.unwrap_or(NB_CONCENTRIC_CIRCLES);
                    }
                    Pattern::ParallelStripes => {
                        nb_pattern = p.nb_parallel_stripes.unwrap_or(NB_PARALLEL_STRIPES);
                        var_stripes = p.var_parallel_stripes.unwrap_or(VAR_PARALLEL_STRIPES);
                    }
                    Pattern::CrossedStripes => {
                        nb_pattern = p.nb_crossed_stripes.unwrap_or(NB_CROSSED_STRIPES);
                        var_stripes = p.var_crossed_stripes.unwrap_or(VAR_CROSSED_STRIPES);
                    }
                    Pattern::ParallelWaves => {
                        nb_pattern = p.nb_parallel_waves.unwrap_or(NB_PARALLEL_WAVES);
                        width_pattern = p.width_wave.unwrap_or(WIDTH_WAVE);
                    }
                    Pattern::ParallelSawteeth => {
                        nb_pattern = p.nb_parallel_sawteeth.unwrap_or(NB_PARALLEL_SAWTEETH);
                        width_pattern = p.width_sawtooth.unwrap_or(WIDTH_SAWTOOTH);
                    }
                }
            } else {
                match pattern {
                    Pattern::FreeCircles => nb_pattern = NB_FREE_CIRCLES,
                    Pattern::FreeTriangles => nb_pattern = NB_FREE_TRIANGLES,
                    Pattern::FreeStripes => {
                        nb_pattern = NB_FREE_STRIPES;
                        width_pattern = WIDTH_STRIPE;
                    }
                    Pattern::FreeSpirals => {
                        nb_pattern = NB_FREE_SPIRALS;
                        width_pattern = WIDTH_SPIRAL;
                        tightness_spiral = TIGHTNESS_SPIRAL;
                    }
                    Pattern::ConcentricCircles => nb_pattern = NB_CONCENTRIC_CIRCLES,
                    Pattern::ParallelStripes => {
                        nb_pattern = NB_PARALLEL_STRIPES;
                        var_stripes = VAR_PARALLEL_STRIPES;
                    }
                    Pattern::CrossedStripes => {
                        nb_pattern = NB_CROSSED_STRIPES;
                        var_stripes = VAR_CROSSED_STRIPES;
                    }
                    Pattern::ParallelWaves => {
                        nb_pattern = NB_PARALLEL_WAVES;
                        width_pattern = WIDTH_WAVE;
                    }
                    Pattern::ParallelSawteeth => {
                        nb_pattern = NB_PARALLEL_SAWTEETH;
                        width_pattern = WIDTH_SAWTOOTH;
                    }
                }
            }
            (nb_pattern, var_stripes, width_pattern, tightness_spiral)
        };

        if themes.is_empty() {
            if colors.is_empty() {
                themes.insert(
                    String::from("-default-"),
                    Chooser::new(vec![(
                        ThemeItem(Color::random(rng), None, None, Salt::none()),
                        BASE_WEIGHT,
                    )]),
                );
            } else {
                themes.insert(
                    String::from("-default-"),
                    Chooser::new(vec![(
                        ThemeItem(
                            *colors
                                .get(*colors.keys().collect::<Vec<_>>().choose(rng).unwrap())
                                .unwrap(),
                            None,
                            None,
                            Salt::none(),
                        ),
                        BASE_WEIGHT,
                    )]),
                );
            }
        }

        // Get tiling-specific options according to picked shapes
        let (size_tiling, nb_delaunay) = {
            if let Some(ConfigData {
                patterns: _,
                tilings: Some(t),
            }) = self.data
            {
                match tiling {
                    Tiling::Hexagons => (t.size_hex.unwrap_or(size), 0),
                    Tiling::Triangles => (t.size_tri.unwrap_or(size), 0),
                    Tiling::HexagonsAndTriangles => (t.size_hex_and_tri.unwrap_or(size), 0),
                    Tiling::SquaresAndTriangles => (t.size_squ_and_tri.unwrap_or(size), 0),
                    Tiling::Rhombus => (t.size_rho.unwrap_or(size), 0),
                    Tiling::Pentagons(_) => (t.size_pen.unwrap_or(size), 0),
                    Tiling::Delaunay => (0.0, t.nb_delaunay.unwrap_or(NB_DELAUNAY)),
                }
            } else {
                match tiling {
                    Tiling::Hexagons => (size, 0),
                    Tiling::Triangles => (size, 0),
                    Tiling::HexagonsAndTriangles => (size, 0),
                    Tiling::SquaresAndTriangles => (size, 0),
                    Tiling::Rhombus => (size, 0),
                    Tiling::Pentagons(_) => (size, 0),
                    Tiling::Delaunay => (0.0, NB_DELAUNAY),
                }
            }
        };
        let (line_width, line_color_default) = {
            if let Some(lines) = self.lines {
                lines.get_settings(tiling, &colors)
            } else {
                (LINE_WIDTH, LINE_COLOR)
            }
        };

        SceneCfg {
            deviation,
            distance,
            theme: themes
                .get(&theme)
                .unwrap_or_else(|| {
                    themes
                        .get(*themes.keys().collect::<Vec<_>>().choose(rng).unwrap())
                        .unwrap()
                })
                .clone(),
            frame: Frame {
                x: 0,
                y: 0,
                w: width,
                h: height,
            },
            tiling,
            line_width,
            line_color: color_from_value(&Value::String(line_color_override), &colors)
                .unwrap_or_else(|_| {
                    color_from_value(&Value::String(line_color_default.to_string()), &colors)
                        .unwrap_or(Color(0, 0, 0))
                }),
            pattern,
            nb_pattern,
            var_stripes,
            nb_delaunay,
            size_tiling,
            width_pattern,
            tightness_spiral,
        }
    }
}

/// Parse a color code: decimal (0-255) or hex (00-FF)
fn color_from_value(val: &Value, dict: &HashMap<String, Color>) -> Result<Color, String> {
    match val {
        Value::String(s) => {
            if let Some(color) = dict.get(s.as_str()) {
                return Ok(*color);
            }
            if s.len() == 7 && &s[0..1] == "#" {
                let r = usize::from_str_radix(&s[1..3], 16);
                let g = usize::from_str_radix(&s[3..5], 16);
                let b = usize::from_str_radix(&s[5..7], 16);
                match (r, g, b) {
                    (Ok(r), Ok(g), Ok(b)) => Ok(Color(r, g, b)),
                    _ => Err(format!(
                        "{:?} is not a valid color format.\nUse [0, 0, 255] or \"#0000FF\"",
                        s
                    )),
                }
            } else {
                Err(format!(
                    "{:?} is not a valid color format.\nUse [0, 0, 255] or \"#0000FF\"",
                    s
                ))
            }
        }
        Value::Array(arr) => {
            if arr.len() != 3 {
                return Err(format!(
                    "{:?} is not a valid color format.\nUse [0, 0, 255] or \"#0000FF\"",
                    val
                ));
            }
            match &arr[0..3] {
                [Value::Integer(r), Value::Integer(g), Value::Integer(b)] => {
                    Ok(Color(*r as usize, *g as usize, *b as usize))
                }
                _ => Err(format!(
                    "{:?} is not a valid color format.\nUse [0, 0, 255] or \"#0000FF\"",
                    arr
                )),
            }
        }
        _ => Err(format!(
            "{:?} is not a valid color format.\nUse [0, 0, 255] or \"#0000FF\"",
            val
        )),
    }
}

fn theme_item_from_value(val: &Value, dict: &HashMap<String, Color>) -> (ThemeItem, usize) {
    let warn_invalid = |_x| {};
    match val {
        Value::String(s) => {
            let mut color = Color(0, 0, 0);
            let mut wht = BASE_WEIGHT;
            let mut var = None;
            let mut dist = None;
            for item in s.split(' ') {
                if item.is_empty() {
                    continue;
                }
                if &item[0..1] == "x" {
                    wht = item[1..].parse().unwrap_or(BASE_WEIGHT);
                } else if &item[0..1] == "~" {
                    var = item[1..]
                        .parse::<usize>()
                        .map(Some)
                        .unwrap_or_else(|_| None);
                } else if &item[0..1] == "!" {
                    dist = item[1..]
                        .parse::<usize>()
                        .map(Some)
                        .unwrap_or_else(|_| None);
                } else {
                    match color_from_value(&Value::String(item.to_string()), dict) {
                        Ok(c) => color = c,
                        Err(e) => {
                            warn_invalid(e);
                        }
                    }
                }
            }
            (ThemeItem(color, var, dist, Salt::none()), wht)
        }
        Value::Table(map) => {
            let color = match map.get("color") {
                Some(val) => match color_from_value(val, dict) {
                    Ok(c) => c,
                    Err(e) => {
                        warn_invalid(e);
                        Color(0, 0, 0)
                    }
                },
                None => Color(0, 0, 0),
            };
            let var = (match map.get("variability") {
                Some(Value::Integer(v)) => Some(*v),
                Some(Value::Float(v)) => Some(v.round() as i64),
                Some(_x) => None,
                None => None,
            })
            .map(|n| n.max(0) as usize);
            let dist = (match map.get("distance") {
                Some(Value::Integer(d)) => Some(*d),
                Some(Value::Float(d)) => Some(d.round() as i64),
                Some(_x) => None,
                None => None,
            })
            .map(|n| n.max(0) as usize);
            let wht = match map.get("weight") {
                Some(Value::Integer(w)) => *w.max(&0) as usize,
                Some(Value::Float(w)) => w.round().max(0.0) as usize,
                Some(_x) => BASE_WEIGHT,
                None => BASE_WEIGHT,
            };
            let salt = match map.get("salt") {
                None => Salt::none(),
                Some(Value::Array(vec)) => {
                    let mut salt = Salt::default();
                    for item in vec.iter() {
                        if let Value::Table(tbl) = item {
                            let color = tbl
                                .get("color")
                                .map(|v| color_from_value(v, dict).unwrap_or(Color(0, 0, 0)))
                                .unwrap_or(Color(0, 0, 0));
                            let likeliness = match tbl.get("likeliness") {
                                None => 1.0,
                                Some(Value::Float(f)) => *f,
                                Some(Value::Integer(n)) => *n as f64,
                                Some(_v) => 1.0,
                            };
                            let variability = match tbl.get("variability") {
                                None => 0,
                                Some(Value::Integer(n)) => {
                                    if *n > 0 {
                                        *n as usize
                                    } else {
                                        0
                                    }
                                }
                                Some(Value::Float(f)) => {
                                    if *f > 0. {
                                        f.round() as usize
                                    } else {
                                        0
                                    }
                                }
                                Some(_v) => 0,
                            };
                            salt.0.push(SaltItem {
                                color,
                                likeliness,
                                variability,
                            });
                        }
                    }
                    salt
                }
                _ => Salt::none(),
            };
            (ThemeItem(color, var, dist, salt), wht)
        }
        val => {
            warn_invalid(val.to_string());
            (
                ThemeItem(Color(0, 0, 0), None, None, Salt::none()),
                BASE_WEIGHT,
            )
        }
    }
}

/// Read group of colors as a theme
fn theme_from_value(
    v: &Value,
    colors: &ColorList,
    themes: &ThemeList,
) -> Result<Chooser<ThemeItem>, String> {
    let mut items = Vec::new();
    if let Value::String(s) = v {
        if let Some(th) = themes.get(s) {
            items = th.extract();
        }
    }
    match v {
        Value::Array(a) => {
            for x in a {
                if let Value::String(s) = x {
                    if let Some(th) = themes.get(s) {
                        items.append(&mut th.extract());
                        continue;
                    }
                }
                let (item, weight) = theme_item_from_value(x, colors);
                items.push((item, weight));
            }
            Ok(Chooser::new(items))
        }
        _ => Err(format!(
            "{:?} is not a valid theme.
Provide a theme item or an array of theme items",
            v
        )),
    }
}

fn shapes_from_value(
    val: &Value,
    shapes: &HashMap<String, (Chooser<Pattern>, Chooser<Tiling>)>,
) -> (Chooser<Pattern>, Chooser<Tiling>) {
    let mut tilings = Chooser::new(vec![]);
    let mut patterns = Chooser::new(vec![]);
    match val {
        Value::Array(arr) => {
            for x in arr {
                match x {
                    Value::String(s) => {
                        if let Some(sh) = shapes.get(s) {
                            let (p, t) = sh;
                            tilings.append(t.extract());
                            patterns.append(p.extract());
                        } else {
                            add_shape(&s[..], BASE_WEIGHT, &mut tilings, &mut patterns);
                        }
                    }
                    Value::Array(a) => {
                        if a.len() == 2 {
                            match &a[..] {
                                [Value::String(s), Value::Integer(w)] if *w > 0 => {
                                    add_shape(&s[..], *w as usize, &mut tilings, &mut patterns)
                                }
                                _ => println!("{} is not a valid shape.", x),
                            }
                        } else {
                            println!("{} is not a valid shape.", x);
                        }
                    }
                    _ => println!("{} is not a valid shape.", x),
                }
            }
        }
        _ => println!("{} is not an array of shapes.", val),
    }
    (patterns, tilings)
}

/// Read shape from one of its names
fn add_shape(s: &str, w: usize, tilings: &mut Chooser<Tiling>, patterns: &mut Chooser<Pattern>) {
    match s {
        "H" | "hex." | "hexagons" => tilings.push(Tiling::Hexagons, w),
        "T" | "tri." | "triangles" => tilings.push(Tiling::Triangles, w),
        "H&T" | "hex.&tri." | "hexagons&squares" => tilings.push(Tiling::HexagonsAndTriangles, w),
        "S&T" | "squ.&tri." | "squares&triangles" => tilings.push(Tiling::SquaresAndTriangles, w),
        "R" | "rho." | "rhombus" => tilings.push(Tiling::Rhombus, w),
        "D" | "del." | "delaunay" => tilings.push(Tiling::Delaunay, w),
        "P" | "pen." | "pentagons" => tilings.push(Tiling::Pentagons(0), w),
        "P1" | "pen.1" | "pentagons-1" => tilings.push(Tiling::Pentagons(1), w),
        "P2" | "pen.2" | "pentagons-2" => tilings.push(Tiling::Pentagons(2), w),
        "P3" | "pen.3" | "pentagons-3" => tilings.push(Tiling::Pentagons(3), w),
        "P4" | "pen.4" | "pentagons-4" => tilings.push(Tiling::Pentagons(4), w),
        "P5" | "pen.5" | "pentagons-5" => tilings.push(Tiling::Pentagons(5), w),
        "P6" | "pen.6" | "pentagons-6" => tilings.push(Tiling::Pentagons(6), w),
        "FC" | "f-cir." | "free-circles" => patterns.push(Pattern::FreeCircles, w),
        "FT" | "f-tri." | "free-triangles" => patterns.push(Pattern::FreeTriangles, w),
        "FR" | "f-str." | "free-stripes" => patterns.push(Pattern::FreeStripes, w),
        "FP" | "f-spi." | "free-spirals" => patterns.push(Pattern::FreeSpirals, w),
        "CC" | "c-cir." | "concentric-circles" => patterns.push(Pattern::ConcentricCircles, w),
        "PS" | "p-str." | "parallel-stripes" => patterns.push(Pattern::ParallelStripes, w),
        "CS" | "c-str." | "crossed-stripes" => patterns.push(Pattern::CrossedStripes, w),
        "PW" | "p-wav." | "parallel-waves" => patterns.push(Pattern::ParallelWaves, w),
        "PT" | "p-saw." | "parallel-sawteeth" => patterns.push(Pattern::ParallelSawteeth, w),
        _ => println!("{} is not recognized as a shape", s),
    }
}

fn choose_theme_shapes(
    rng: &mut StdRng,
    entry: &Option<Vec<ConfigEntry>>,
    time: u64,
) -> (String, String, String) {
    match entry {
        None => (String::from(""), String::from(""), String::from("")),
        Some(v) => {
            let mut valid = Chooser::new(vec![]);
            for e in v {
                let markers = e
                    .span
                    .as_ref()
                    .unwrap_or(&"-".to_string())
                    .split('-')
                    .map(String::from)
                    .collect::<Vec<_>>();
                let start = markers
                    .get(0)
                    .as_ref()
                    .unwrap_or(&&String::from("0"))
                    .parse::<usize>()
                    .unwrap_or(0);
                let end = markers
                    .get(1)
                    .as_ref()
                    .unwrap_or(&&String::from("2400"))
                    .parse::<usize>()
                    .unwrap_or(2400);
                if start as u64 <= time && time <= end as u64 {
                    valid.push(e, e.distance.unwrap_or(BASE_WEIGHT));
                }
            }
            match valid.choose(rng) {
                None => (String::from(""), String::from(""), String::from("")),
                Some(chosen_entry) => {
                    let chosen_theme = match &chosen_entry.themes {
                        None => String::from(""),
                        Some(th) => th
                            .choose(rng)
                            .map(String::from)
                            .unwrap_or_else(|| String::from("")),
                    };
                    let chosen_shapes = match &chosen_entry.shapes {
                        None => String::from(""),
                        Some(sh) => sh
                            .choose(rng)
                            .map(String::from)
                            .unwrap_or_else(|| String::from("")),
                    };
                    let line_color = chosen_entry
                        .line_color
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| String::from(""));
                    (chosen_theme, chosen_shapes, line_color)
                }
            }
        }
    }
}

impl ConfigLines {
    fn get_settings(&self, tiling: Tiling, colors: &HashMap<String, Color>) -> (f64, Color) {
        let (w, c) = match tiling {
            Tiling::Hexagons => (self.hex_width, &self.hex_color),
            Tiling::Triangles => (self.tri_width, &self.tri_color),
            Tiling::HexagonsAndTriangles => (self.hex_and_tri_width, &self.hex_and_tri_color),
            Tiling::SquaresAndTriangles => (self.squ_and_tri_width, &self.squ_and_tri_color),
            Tiling::Rhombus => (self.rho_width, &self.rho_color),
            Tiling::Pentagons(_) => (self.pen_width, &self.pen_color),
            Tiling::Delaunay => (self.del_width, &self.del_color),
        };
        (
            w.unwrap_or_else(|| self.width.unwrap_or(LINE_WIDTH)),
            match c {
                Some(c) => color_from_value(&Value::String(c.to_string()), colors).ok(),
                None => None,
            }
            .unwrap_or_else(|| {
                match &self.color {
                    Some(color) => color_from_value(&Value::String(color.to_string()), colors).ok(),
                    None => None,
                }
                .unwrap_or(LINE_COLOR)
            }),
        )
    }
}

const DEVIATION: usize = 20;
const DISTANCE: usize = 40;
const SIZE: f64 = 15.;
const WIDTH: usize = 1000;
const HEIGHT: usize = 600;
const NB_FREE_CIRCLES: usize = 10;
const NB_FREE_TRIANGLES: usize = 15;
const NB_FREE_STRIPES: usize = 7;
const NB_PARALLEL_STRIPES: usize = 15;
const NB_CONCENTRIC_CIRCLES: usize = 5;
const NB_CROSSED_STRIPES: usize = 10;
const NB_FREE_SPIRALS: usize = 3;
const NB_PARALLEL_WAVES: usize = 15;
const NB_PARALLEL_SAWTEETH: usize = 15;
const VAR_PARALLEL_STRIPES: usize = 15;
const VAR_CROSSED_STRIPES: usize = 10;
const WIDTH_SPIRAL: f64 = 0.3;
const WIDTH_STRIPE: f64 = 0.1;
const WIDTH_WAVE: f64 = 0.3;
const WIDTH_SAWTOOTH: f64 = 0.3;
const TIGHTNESS_SPIRAL: f64 = 0.5;
const NB_DELAUNAY: usize = 1000;
const LINE_WIDTH: f64 = 1.0;
const LINE_COLOR: Color = Color(0, 0, 0);
