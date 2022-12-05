// CLI dep
use clap::{value_parser, Arg, Command};
// WebP processing deps
use image::*;
use serde_json::Value;
use webp::*;
// General-use deps
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
// Crate internals also used for MapLibre deployment
mod summary;
use crate::summary::*;

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = Command::new("odm_postprocess")
        .about("OpenDroneMap post-processing program for cmoran's drone pipeline")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("height_to_px_res")
                .about("Using the drone's height in meters, calculate the appropriate pixel resolution for NodeODM")
                .arg(
                    Arg::new("height")
                        .short('H')
                        .long("height")
                        .help("Height of the drone in meters")
                        .required(true)
                        .value_parser(value_parser!(f64))
                )
                .arg(
                        Arg::new("y_res")
                            .short('y')
                            .long("y_res")
                            .help("Resolution of a photo in pixels along the y-axis (vertical)")
                            .required(true)
                            .value_parser(value_parser!(usize))
                )
                .arg(
                    Arg::new("x_res")
                        .short('x')
                        .long("x_res")
                        .help("Resolution of a photo in pixels along the x-axis (horizontal)")
                        .required(true)
                        .value_parser(value_parser!(usize))
            )
        )
        .subcommand(
            Command::new("convert")
                .about("Convert an ODM post-process package to an upload-able package for a MapLibre site")
                .arg(
                    Arg::new("input-dir")
                        .short('i')
                        .long("input-dir")
                        .help("<input-directory>")
                        .required(true)
                        .value_parser(value_parser!(std::path::PathBuf))
                )
                .arg(
                    Arg::new("output-dir")
                        .short('o')
                        .long("output-dir")
                        .help("<output-directory>")
                        .required(true)
                        .value_parser(value_parser!(std::path::PathBuf))
                )
                .arg(
                    Arg::new("size-factor")
                        .short('s')
                        .long("size-factor")
                        .help("Scale down from the original orthophoto [0.0-1.0]")
                        .required(false)
                        .default_value("0.2")
                        .value_parser(value_parser!(f64))
                )
                .arg(
                    Arg::new("webp-quality")
                        .short('q')
                        .long("webp-quality")
                        .help("Quality of the WebP compression")
                        .required(false)
                        .default_value("90.0")
                        .value_parser(value_parser!(f32))
                )

        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("height_to_px_res", h2pxr_matches)) => {
            let height = *h2pxr_matches.get_one::<f64>("height").unwrap();
            let y_res = *h2pxr_matches.get_one::<usize>("y_res").unwrap();
            let x_res = *h2pxr_matches.get_one::<usize>("x_res").unwrap();

            // TO_DO

            println!("{:?},{},{}", height, x_res, y_res);
        }
        Some(("convert", convert_matches)) => {
            let input_dir = convert_matches
                .get_one::<PathBuf>("input-dir")
                .unwrap()
                .to_owned();
            let output_dir = convert_matches
                .get_one::<PathBuf>("output-dir")
                .unwrap()
                .to_owned();
            let size_factor = *convert_matches.get_one::<f64>("size-factor").unwrap();
            let quality = *convert_matches.get_one::<f32>("webp-quality").unwrap();

            // <-- CONVERT THE ORIGINAL ODM OUTPUT TO A WEBMAP-COMPATIBLE VERSION -->
            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir).unwrap();
            }
        
            let input_json_path = input_dir
                .join("odm_georeferencing")
                .join("odm_georeferenced_model")
                .with_extension("info.json");

            println!("json_path: {:?}", input_json_path);
            let (bounds, center) = get_bounds(input_json_path);
            println!("Bounds: {:?}", bounds);
        
            let summary = Summary {
                title: "".to_string(),
                description: "".to_string(),
                bounds,
                center,
            };
            let j = serde_json::to_string_pretty(&summary).unwrap();
            let path_summary_json = output_dir.join("summary").with_extension("json");
            let mut summary_json = fs::File::create(path_summary_json).unwrap();
            summary_json.write_all(j.as_bytes()).unwrap();
        
            process_orthophoto(input_dir, output_dir, size_factor, quality).unwrap();
        }
        _ => (),
    }

    Ok(())
}

/// Process the full-res ODM orthophoto package into a smaller web-compatible product
fn process_orthophoto(
    input_dir: PathBuf,
    output_dir: PathBuf,
    size_factor: f64,
    quality: f32,
) -> Result<(), Box<dyn Error>> {
    let path_orthophoto = Path::new("odm_orthophoto").join("odm_orthophoto.png");
    let input_path_orthophoto = input_dir.join(&path_orthophoto);

    // Create a low-definition .webp version of the orthophoto .png
    let mut rdr = image::io::Reader::open(&input_path_orthophoto)?.with_guessed_format()?;
    // Orthographic .png mosaics routinely become too large for the default Reader
    // We need to explicitly de-limit the Reader to accommodate those imagers
    rdr.no_limits();

    let img = match rdr.decode() {
        Ok(img) => img,
        Err(e) => panic!("Error: {:?} on {:?}", e, &input_path_orthophoto),
    };
    let (w, h) = img.dimensions();
    // Save the full-def orthographic photo
    img.save(output_dir.join("odm_orthophoto").with_extension("png"))
        .unwrap();

    let img = image::DynamicImage::ImageRgba8(imageops::resize(
        &img,
        (w as f64 * size_factor) as u32,
        (h as f64 * size_factor) as u32,
        imageops::FilterType::Triangle,
    ));

    let encoder: Encoder = Encoder::from_image(&img)?;
    let webp: WebPMemory = encoder.encode(quality);

    let output_path_orthophoto = output_dir.join("odm_orthophoto").with_extension("webp");

    fs::write(&output_path_orthophoto, &*webp)?;

    Ok(())
}

/// Get orthophoto bounds from the ODM JSON summary
pub fn get_bounds(json_path: PathBuf) -> (Bounds, Center) {
    let json = std::fs::read_to_string(json_path).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    println!("{}", v["stats"]["bbox"]["EPSG:4326"]["bbox"]);

    let min_x = v["stats"]["bbox"]["EPSG:4326"]["bbox"]["minx"]
        .to_string()
        .parse::<f32>()
        .unwrap();
    let min_y = v["stats"]["bbox"]["EPSG:4326"]["bbox"]["miny"]
        .to_string()
        .parse::<f32>()
        .unwrap();
    let max_x = v["stats"]["bbox"]["EPSG:4326"]["bbox"]["maxx"]
        .to_string()
        .parse::<f32>()
        .unwrap();
    let max_y = v["stats"]["bbox"]["EPSG:4326"]["bbox"]["maxy"]
        .to_string()
        .parse::<f32>()
        .unwrap();

    let bounds = Bounds {
        min_x,
        max_x,
        min_y,
        max_y,
    };

    let center = Center {
        lat: (bounds.max_y + bounds.min_y) / 2.0,
        lon: (bounds.max_x + bounds.min_x) / 2.0,
    };

    (bounds, center)
}
