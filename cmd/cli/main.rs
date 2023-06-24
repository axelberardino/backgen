use backgen::deserializer::MetaConfig;
use backgen::scene::Scene;
use backgen::svg::*;
use chrono::{Local, Timelike};
use clap::{arg, Parser};
use image::GenericImageView;
use rand::{rngs::StdRng, SeedableRng};

// CLI flags configuration
#[derive(Parser)]
#[clap(name = "backgen")]
#[clap(about = "BackGen CLI", long_about = None)]
struct Cli {
    /// Id from which an image should be generated (default = random)
    #[arg(default_value = None)]
    #[arg(long, value_name = "id")]
    id: Option<u64>,

    /// Output files
    #[arg(default_value = "output.png")]
    #[arg(long, value_name = "output")]
    output: String,
}

fn main() {
    let args = Cli::parse();

    let id = args.id.unwrap_or_else(|| {
        let now = Local::now();
        let h = now.hour();
        let m = now.minute();
        (h * 100 + m) as u64
    });
    let dest = args.output;

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

    let output_file = dest.clone() + ".output.png";
    document.save(&output_file).expect("can't save document");

    let img = image::open(output_file).expect("can't open png image");
    let (width, height) = img.dimensions();
    let blurhash = blurhash::encode(4, 3, width, height, &img.into_rgba8().into_vec());
    let pixels = blurhash::decode(blurhash.as_str(), width, height, 1.2);

    println!("blurhash is: {blurhash}");

    image::save_buffer(
        dest + ".blur.png",
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .expect("can't save image");
}
