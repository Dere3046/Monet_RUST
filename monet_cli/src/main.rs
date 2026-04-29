use image::open;
use material_colors::color::Argb;
use material_colors::palette::TonalPalette;
use material_colors::theme::{Theme, ThemeBuilder};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;
use wallpaper_colors::{ScoringStrategy, WallpaperColors};

#[derive(Serialize)]
struct ThemeOutput {
    seed_color: String,
    seed_color_argb: u32,
    color_hints: String,
    light: HashMap<String, String>,
    dark: HashMap<String, String>,
    palettes: PalettesOutput,
}

#[derive(Serialize)]
struct PalettesOutput {
    primary: Vec<String>,
    secondary: Vec<String>,
    tertiary: Vec<String>,
    neutral: Vec<String>,
    neutral_variant: Vec<String>,
    error: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: monet <wallpaper_path> [options]");
        eprintln!("  --strategy material|java   Scoring strategy (default: material)");
        eprintln!("  --output, -o <file>        Save JSON output to file instead of stdout");
        eprintln!("  --seed-only                Output only the seed color hex");
        process::exit(1);
    }

    let image_path = &args[1];
    let strategy = parse_strategy(&args);
    let output_path = parse_output(&args);
    let seed_only = parse_seed_only(&args);

    let img = match open(image_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
            process::exit(1);
        }
    };

    let wallpaper_colors = WallpaperColors::from_bitmap_with_strategy(img, 0.0, strategy);
    let seed = *wallpaper_colors.primary_color();

    if seed_only {
        let output = format!("{}\n", seed.to_hex_with_pound());
        if let Some(path) = output_path {
            if let Err(e) = fs::write(path, &output) {
                eprintln!("Failed to write output file: {}", e);
                process::exit(1);
            }
        } else {
            print!("{}", output);
        }
        return;
    }

    let seed_u32 = argb_to_u32(seed);
    let hints = format!("{:?}", wallpaper_colors.color_hints());

    eprintln!("Seed color: {}", seed.to_hex_with_pound());
    eprintln!("Seed color ARGB: 0x{:08x}", seed_u32);
    eprintln!("Color hints: {}", hints);

    let theme = ThemeBuilder::with_source(seed).build();

    let output = build_output(seed, seed_u32, hints, theme);
    let json = serde_json::to_string_pretty(&output).unwrap();

    if let Some(path) = output_path {
        if let Err(e) = fs::write(path, json) {
            eprintln!("Failed to write output file: {}", e);
            process::exit(1);
        }
        eprintln!("Output saved to: {}", path);
    } else {
        println!("{}", json);
    }
}

fn parse_strategy(args: &[String]) -> ScoringStrategy {
    for i in 0..args.len() {
        if args[i] == "--strategy" && i + 1 < args.len() {
            return match args[i + 1].as_str() {
                "java" => ScoringStrategy::JavaOriginal,
                _ => ScoringStrategy::MaterialColors,
            };
        }
    }
    ScoringStrategy::MaterialColors
}

fn parse_output(args: &[String]) -> Option<&str> {
    for i in 0..args.len() {
        if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
            return Some(&args[i + 1]);
        }
    }
    None
}

fn parse_seed_only(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--seed-only")
}

fn build_output(seed: Argb, seed_u32: u32, hints: String, theme: Theme) -> ThemeOutput {
    let light: HashMap<String, String> = theme
        .schemes
        .light
        .into_iter()
        .map(|(k, v)| (k, v.to_hex_with_pound()))
        .collect();
    let dark: HashMap<String, String> = theme
        .schemes
        .dark
        .into_iter()
        .map(|(k, v)| (k, v.to_hex_with_pound()))
        .collect();

    ThemeOutput {
        seed_color: seed.to_hex_with_pound(),
        seed_color_argb: seed_u32,
        color_hints: hints,
        light,
        dark,
        palettes: PalettesOutput {
            primary: extract_palette_tones(&theme.palettes.primary),
            secondary: extract_palette_tones(&theme.palettes.secondary),
            tertiary: extract_palette_tones(&theme.palettes.tertiary),
            neutral: extract_palette_tones(&theme.palettes.neutral),
            neutral_variant: extract_palette_tones(&theme.palettes.neutral_variant),
            error: extract_palette_tones(&theme.palettes.error),
        },
    }
}

fn argb_to_u32(argb: Argb) -> u32 {
    ((argb.alpha as u32) << 24)
        | ((argb.red as u32) << 16)
        | ((argb.green as u32) << 8)
        | (argb.blue as u32)
}

fn extract_palette_tones(palette: &TonalPalette) -> Vec<String> {
    let tones = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 99, 100];
    tones.iter().map(|&t| palette.tone(t).to_hex_with_pound()).collect()
}
