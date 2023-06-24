use image::GenericImageView;
use rand::{rngs::StdRng, Rng, SeedableRng};
use thiserror::Error;

use crate::{deserializer::MetaConfig, scene::Scene, svg::Document};

#[derive(Error, Debug)]
pub enum GenImagError {
    #[error("can't save the generated image: {0}")]
    CantSaveGeneratedImage(#[from] std::io::Error),
    #[error("can't open image: {0}")]
    CantOpenImage(#[from] image::ImageError),
}

/// Generate an image and its blurashs counterpart, from a given id.
/// If no id is given, then a random one is computed.
///
/// # Errors
///
/// Failed if image can't be written, read or generated
pub fn generate_images(
    id: Option<u64>,
    gen_dest: &str,
    blur_dest: &str,
) -> Result<String, GenImagError> {
    let id = id.unwrap_or_else(|| {
        let mut rng = rand::thread_rng();
        rng.gen()
    });

    let mut rng = StdRng::seed_from_u64(id);
    let cfg = MetaConfig::from_string(String::new()).pick_cfg(&mut rng, id);
    let scene = Scene::new(&cfg, &mut rng);
    let stroke = cfg.line_color;
    let stroke_width = cfg.line_width;
    let stroke_like_fill = stroke_width < 0.0001;

    // Generate document
    let mut document = Document::new(cfg.frame);
    for (pos, elem) in cfg.make_tiling(&mut rng) {
        let fill = scene.color(pos, &mut rng);
        document.add(
            elem.with_fill_color(fill)
                .with_stroke_color(if stroke_like_fill { fill } else { stroke })
                .with_stroke_width(stroke_width.max(0.1)),
        );
    }

    document.save(gen_dest)?;

    let img = image::open(gen_dest)?;
    let (width, height) = img.dimensions();
    let blurhash = blurhash::encode(4, 3, width, height, &img.into_rgba8().into_vec());
    let pixels = blurhash::decode(blurhash.as_str(), width, height, 1.2);

    image::save_buffer(blur_dest, &pixels, width, height, image::ColorType::Rgba8)?;

    Ok(blurhash)
}
