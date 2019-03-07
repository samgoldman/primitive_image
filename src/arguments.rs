extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Primitive", about = "Generate SVG approximations of images!", author = "Sam Goldman", rename_all = "kebab-case")]
pub struct Opt {
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    /// Path to the image to be approximated (.jpg, .png, .tif, .gif, or .bmp)
    pub in_path: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    /// Path to the output file (.jpg, .png, .bmp, .ico, .gif, or .svg)
    pub out_path: PathBuf,

    #[structopt(short = "n", default_value = "100")]
    /// Number of polygons to use
    pub n: u32,

    #[structopt(long, default_value = "100")]
    /// Maximum age for each hill climbing attempt
    pub max_age: u32,

    #[structopt(long, default_value = "100")]
    /// The value to scale the image's largest dimension to. <= 0 prevents scaling
    pub scale_to: u32,

    #[structopt(long, default_value = "0")]
    /// The random seed. 0 picks a seed based on the time
    pub seed: u64,

    #[structopt(short, parse(from_occurrences))]
    /// Turn on verbosity (use multiple for different levels)
    pub v: usize
}