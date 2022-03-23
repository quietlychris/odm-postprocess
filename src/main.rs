use image::*;
use serde_json::Value;
use webp::*;

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

mod summary;
use crate::summary::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_dir = args[1].parse::<PathBuf>().unwrap();
    let output_dir = args[2].parse::<PathBuf>().unwrap();

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).unwrap();
    }

    let input_json_path = input_dir
        .join("odm_georeferencing")
        .join("odm_georeferenced_model")
        .with_extension("info.json");
    // .with_extension("json");
    println!("json_path: {:?}", input_json_path);
    let (bounds, center) = get_bounds(input_json_path);
    println!("Bounds: {:?}", bounds);

    let summary  = Summary {title: "".to_string(), description: "".to_string(), bounds, center };
    let j = serde_json::to_string_pretty(&summary).unwrap();
    let path_summary_json = output_dir.join("summary").with_extension("json");
    let mut summary_json = fs::File::create(path_summary_json).unwrap(); 
    summary_json.write_all(j.as_bytes()).unwrap();   

    process_orthophoto(&input_dir, &output_dir).unwrap();
}

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

fn process_orthophoto(input_dir: &Path, output_dir: &Path) -> Result<(), Box<dyn Error>> {

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
    let size_factor = 0.2;
    // Save the full-def orthographic photo
    img.save(output_dir.join("odm_orthophoto").with_extension("png")).unwrap();

    let img = image::DynamicImage::ImageRgba8(imageops::resize(
        &img,
        (w as f64 * size_factor) as u32,
        (h as f64 * size_factor) as u32,
        imageops::FilterType::Triangle,
    ));

    let encoder: Encoder = Encoder::from_image(&img)?;
    let webp_quality = 90.0;
    let webp: WebPMemory = encoder.encode(webp_quality);

    let output_path_orthophoto = output_dir.join("odm_orthophoto").with_extension("webp");

    fs::write(&output_path_orthophoto, &*webp)?;

    Ok(())
}


