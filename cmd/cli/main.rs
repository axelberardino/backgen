use backgen::gen_image::generate_images;
use clap::{arg, Parser};

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
