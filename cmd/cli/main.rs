use backgen::deserializer::MetaConfig;
use backgen::scene::Scene;
use backgen::svg::*;
use chrono::{Local, Timelike};
use clap::{arg, Parser};
use image::GenericImageView;
use rand::{rngs::StdRng, SeedableRng};
use thiserror::Error;

// CLI flags configuration
#[derive(Parser)]
#[clap(name = "backgen")]
#[clap(about = "BackGen CLI", long_about = None)]
struct Cli {
    /// Id from which an image should be generated (default = random)
    #[arg(default_value = None)]
    #[arg(long, value_name = "id")]
    id: Option<u64>,

    /// Generated image output destination file
    #[arg(default_value = "gen_output.png")]
    #[arg(long, value_name = "generated_output")]
    gen_dest: String,

    /// Blur image output destination file
    #[arg(default_value = "blur_output.png")]
    #[arg(long, value_name = "blur_output")]
    blur_dest: String,
}

fn main() {
    let args = Cli::parse();

    match generate_images(args.id, &args.gen_dest, &args.blur_dest) {
        Ok(blurhash) => println!("blurhash is: {blurhash}"),
        Err(err) => eprintln!("Error occured: {err}"),
    }
}

#[derive(Error, Debug)]
pub enum GenImagError {
    #[error("can't save the generated image: {0}")]
    CantSaveGeneratedImage(#[from] std::io::Error),
    #[error("can't open image: {0}")]
    CantOpenImage(#[from] image::ImageError),
}

fn generate_images(
    id: Option<u64>,
    gen_dest: &str,
    blur_dest: &str,
) -> Result<String, GenImagError> {
    let id = id.unwrap_or_else(|| {
        let now = Local::now();
        let h = now.hour();
        let m = now.minute();
        (h * 100 + m) as u64
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
