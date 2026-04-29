use crate::color_utils::{composite_colors, set_alpha_component};
use crate::contrast::{calculate_contrast, calculate_luminance};
use image::{DynamicImage, GenericImageView, Rgba};
use indexmap::IndexMap;
use material_colors::color::Argb;
use material_colors::hct::cam16::Cam16;
use material_colors::score::Score;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

const MAX_BITMAP_SIZE: i32 = 112;
const MAX_WALLPAPER_EXTRACTION_AREA: i32 = MAX_BITMAP_SIZE * MAX_BITMAP_SIZE;
const BRIGHT_IMAGE_MEAN_LUMINANCE: f64 = 0.70;
const DARK_THEME_MEAN_LUMINANCE: f64 = 0.30;
const DARK_PIXEL_CONTRAST: f64 = 5.5;
const MAX_DARK_AREA: f64 = 0.05;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ColorHints: i32 {
        const SUPPORTS_DARK_TEXT = 1 << 0;
        const SUPPORTS_DARK_THEME = 1 << 1;
        const FROM_BITMAP = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringStrategy {
    MaterialColors,
    JavaOriginal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WallpaperColors {
    main_colors: Vec<Argb>,
    all_colors: HashMap<u32, u32>,
    color_hints: ColorHints,
}

impl WallpaperColors {
    pub fn from_bitmap(bitmap: DynamicImage) -> Self {
        Self::from_bitmap_with_strategy(bitmap, 0.0, ScoringStrategy::MaterialColors)
    }

    pub fn from_bitmap_java(bitmap: DynamicImage) -> Self {
        Self::from_bitmap_with_strategy(bitmap, 0.0, ScoringStrategy::JavaOriginal)
    }

    pub fn from_bitmap_with_strategy(
        bitmap: DynamicImage,
        dim_amount: f32,
        strategy: ScoringStrategy,
    ) -> Self {
        let (width, height) = bitmap.dimensions();
        let bitmap_area = (width * height) as i32;

        let scaled_bitmap = if bitmap_area > MAX_WALLPAPER_EXTRACTION_AREA as i32 {
            let optimal_size = calculate_optimal_size(width as i32, height as i32);
            bitmap.resize(
                optimal_size.0 as u32,
                optimal_size.1 as u32,
                image::imageops::FilterType::Nearest,
            )
        } else {
            bitmap
        };

        let mut color_to_population: IndexMap<Argb, u32, BuildHasherDefault<ahash::AHasher>> =
            IndexMap::with_hasher(BuildHasherDefault::default());

        let (scaled_width, scaled_height) = scaled_bitmap.dimensions();
        for y in 0..scaled_height {
            for x in 0..scaled_width {
                let Rgba([r, g, b, a]) = scaled_bitmap.get_pixel(x, y);
                let argb = Argb::new(a, r, g, b);
                *color_to_population.entry(argb).or_insert(0) += 1;
            }
        }

        let scored_colors = match strategy {
            ScoringStrategy::MaterialColors => score_material_colors(&color_to_population),
            ScoringStrategy::JavaOriginal => score_java_original(&color_to_population),
        };

        let hints = calculate_dark_hints(&scaled_bitmap, dim_amount);

        let mut main_colors = Vec::new();
        for color in scored_colors.iter().take(3) {
            main_colors.push(*color);
        }

        let mut all_colors = HashMap::new();
        for (color, population) in color_to_population.iter() {
            let color_u32 = argb_to_u32(*color);
            all_colors.insert(color_u32, *population);
        }

        let mut color_hints = ColorHints::FROM_BITMAP;
        color_hints.set(hints, true);

        WallpaperColors {
            main_colors,
            all_colors,
            color_hints,
        }
    }

    pub fn from_color_map(
        color_to_population: HashMap<u32, u32>,
        color_hints: ColorHints,
    ) -> Self {
        Self::from_color_map_with_strategy(
            color_to_population,
            color_hints,
            ScoringStrategy::MaterialColors,
        )
    }

    pub fn from_color_map_with_strategy(
        color_to_population: HashMap<u32, u32>,
        color_hints: ColorHints,
        strategy: ScoringStrategy,
    ) -> Self {
        let mut color_to_population_indexed: IndexMap<
            Argb,
            u32,
            BuildHasherDefault<ahash::AHasher>,
        > = IndexMap::with_hasher(BuildHasherDefault::default());

        for (&color_u32, &population) in color_to_population.iter() {
            let argb = u32_to_argb(color_u32);
            color_to_population_indexed.insert(argb, population);
        }

        let scored_colors = match strategy {
            ScoringStrategy::MaterialColors => score_material_colors(&color_to_population_indexed),
            ScoringStrategy::JavaOriginal => score_java_original(&color_to_population_indexed),
        };

        let mut main_colors = Vec::new();
        for color in scored_colors.iter().take(3) {
            main_colors.push(*color);
        }

        WallpaperColors {
            main_colors,
            all_colors: color_to_population,
            color_hints,
        }
    }

    pub fn primary_color(&self) -> &Argb {
        &self.main_colors[0]
    }

    pub fn secondary_color(&self) -> Option<&Argb> {
        self.main_colors.get(1)
    }

    pub fn tertiary_color(&self) -> Option<&Argb> {
        self.main_colors.get(2)
    }

    pub fn main_colors(&self) -> &[Argb] {
        &self.main_colors
    }

    pub fn all_colors(&self) -> &HashMap<u32, u32> {
        &self.all_colors
    }

    pub fn color_hints(&self) -> ColorHints {
        self.color_hints
    }
}

fn score_material_colors(
    color_to_population: &IndexMap<Argb, u32, BuildHasherDefault<ahash::AHasher>>,
) -> Vec<Argb> {
    Score::score(color_to_population, Some(4), None, Some(true))
}

fn score_java_original(
    color_to_population: &IndexMap<Argb, u32, BuildHasherDefault<ahash::AHasher>>,
) -> Vec<Argb> {
    let mut color_to_cam: HashMap<u32, Cam16> = HashMap::new();
    let mut color_to_pop_u32: HashMap<u32, u32> = HashMap::new();

    for (&argb, &population) in color_to_population.iter() {
        let color_u32 = argb_to_u32(argb);
        let cam: Cam16 = argb.into();
        color_to_cam.insert(color_u32, cam);
        color_to_pop_u32.insert(color_u32, population);
    }

    let hue_proportions = hue_proportions_cam(&color_to_cam, &color_to_pop_u32);
    let color_to_hue_proportion =
        color_to_hue_proportion_cam(&color_to_pop_u32.keys().copied().collect(), &color_to_cam, &hue_proportions);

    let mut color_to_score: Vec<(u32, f64)> = Vec::new();
    for (&color, &proportion) in color_to_hue_proportion.iter() {
        let cam = color_to_cam.get(&color).unwrap();
        let score = score_cam(cam, proportion);
        color_to_score.push((color, score));
    }

    color_to_score.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut colors_by_score_descending: Vec<u32> = Vec::new();
    for (color, _) in color_to_score {
        colors_by_score_descending.push(color);
    }

    let mut main_color_ints: Vec<u32> = Vec::new();
    'find_seed_color: for &color in &colors_by_score_descending {
        let cam = color_to_cam.get(&color).unwrap();
        for &other_color in &main_color_ints {
            let other_cam = color_to_cam.get(&other_color).unwrap();
            if hue_diff(cam, other_cam) < 15.0 {
                continue 'find_seed_color;
            }
        }
        main_color_ints.push(color);
        if main_color_ints.len() >= 4 {
            break;
        }
    }

    let mut main_colors: Vec<Argb> = Vec::new();
    for color_int in main_color_ints {
        main_colors.push(u32_to_argb(color_int));
    }

    main_colors
}

fn score_cam(cam: &Cam16, proportion: f64) -> f64 {
    cam.chroma + (proportion * 100.0)
}

fn hue_diff(a: &Cam16, b: &Cam16) -> f64 {
    let diff = (a.hue - b.hue).abs();
    180.0 - (diff - 180.0).abs()
}

fn hue_proportions_cam(color_to_cam: &HashMap<u32, Cam16>, color_to_population: &HashMap<u32, u32>) -> [f64; 360] {
    let mut proportions = [0.0; 360];
    let total_population: u32 = color_to_population.values().sum();
    let total_population_f = total_population as f64;

    if total_population_f == 0.0 {
        return proportions;
    }

    for (&color, &population) in color_to_population.iter() {
        let cam = color_to_cam.get(&color).unwrap();
        let hue = wrap_degrees(cam.hue.round() as i32);
        proportions[hue as usize] += population as f64 / total_population_f;
    }

    proportions
}

fn color_to_hue_proportion_cam(
    colors: &Vec<u32>,
    color_to_cam: &HashMap<u32, Cam16>,
    hue_proportions: &[f64; 360],
) -> HashMap<u32, f64> {
    let mut result = HashMap::new();

    for &color in colors {
        let cam = color_to_cam.get(&color).unwrap();
        let hue = wrap_degrees(cam.hue.round() as i32);
        let mut proportion = 0.0;
        for i in (hue - 15)..(hue + 15) {
            proportion += hue_proportions[wrap_degrees(i) as usize];
        }
        result.insert(color, proportion);
    }

    result
}

fn wrap_degrees(degrees: i32) -> i32 {
    ((degrees % 360) + 360) % 360
}

fn calculate_optimal_size(width: i32, height: i32) -> (i32, i32) {
    let requested_area = width * height;
    let scale = if requested_area > MAX_WALLPAPER_EXTRACTION_AREA {
        f64::sqrt(MAX_WALLPAPER_EXTRACTION_AREA as f64 / requested_area as f64)
    } else {
        1.0
    };

    let new_width = (width as f64 * scale) as i32;
    let new_height = (height as f64 * scale) as i32;

    if new_width == 0 {
        return (1, new_height.max(1));
    }
    if new_height == 0 {
        return (new_width.max(1), 1);
    }

    (new_width, new_height)
}

fn calculate_dark_hints(source: &DynamicImage, dim_amount: f32) -> ColorHints {
    let (width, height) = source.dimensions();
    let pixel_count = (width * height) as usize;
    let max_dark_pixels = (pixel_count as f64 * MAX_DARK_AREA) as usize;

    let dim_amount = dim_amount.clamp(0.0, 1.0);
    let dimming_layer_alpha = (255.0 * dim_amount) as u8;
    let black_transparent = set_alpha_component(Argb::new(255, 0, 0, 0), dimming_layer_alpha);

    let mut total_luminance = 0.0;
    let mut dark_pixels = 0;

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, a]) = source.get_pixel(x, y);
            let pixel_color = Argb::new(a, r, g, b);

            let composite = composite_colors(black_transparent, pixel_color);
            let adjusted_luminance = calculate_luminance(composite);

            let satisfies_text_contrast =
                calculate_contrast(pixel_color, Argb::new(255, 0, 0, 0)) > DARK_PIXEL_CONTRAST;

            if !satisfies_text_contrast && a != 0 {
                dark_pixels += 1;
            }

            total_luminance += adjusted_luminance;
        }
    }

    let mut hints = ColorHints::empty();
    let mean_luminance = total_luminance / pixel_count as f64;

    if mean_luminance > BRIGHT_IMAGE_MEAN_LUMINANCE && dark_pixels <= max_dark_pixels {
        hints |= ColorHints::SUPPORTS_DARK_TEXT;
    }

    if mean_luminance < DARK_THEME_MEAN_LUMINANCE {
        hints |= ColorHints::SUPPORTS_DARK_THEME;
    }

    hints
}

fn argb_to_u32(argb: Argb) -> u32 {
    ((argb.alpha as u32) << 24)
        | ((argb.red as u32) << 16)
        | ((argb.green as u32) << 8)
        | (argb.blue as u32)
}

fn u32_to_argb(value: u32) -> Argb {
    Argb::new(
        ((value >> 24) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_test_image() -> DynamicImage {
        let mut img = ImageBuffer::new(10, 10);
        for (x, _y, pixel) in img.enumerate_pixels_mut() {
            if x < 5 {
                *pixel = Rgba([255, 0, 0, 255]);
            } else {
                *pixel = Rgba([0, 0, 255, 255]);
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    fn create_solid_color_image(color: Argb, width: u32, height: u32) -> DynamicImage {
        let mut img = ImageBuffer::new(width, height);
        for (_, _, pixel) in img.enumerate_pixels_mut() {
            *pixel = Rgba([color.red, color.green, color.blue, color.alpha]);
        }
        DynamicImage::ImageRgba8(img)
    }

    fn create_gradient_image(width: u32, height: u32) -> DynamicImage {
        let mut img = ImageBuffer::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = 128;
            *pixel = Rgba([r, g, b, 255]);
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_from_bitmap_material_colors() {
        let img = create_test_image();
        let colors = WallpaperColors::from_bitmap(img);
        assert!(!colors.main_colors.is_empty());
        assert!(colors.color_hints.contains(ColorHints::FROM_BITMAP));
    }

    #[test]
    fn test_from_bitmap_java() {
        let img = create_test_image();
        let colors = WallpaperColors::from_bitmap_java(img);
        assert!(!colors.main_colors.is_empty());
        assert!(colors.color_hints.contains(ColorHints::FROM_BITMAP));
    }

    #[test]
    fn test_scoring_strategies_produce_different_results() {
        let img = create_test_image();
        let material = WallpaperColors::from_bitmap(img.clone());
        let java = WallpaperColors::from_bitmap_java(img);
        assert!(!material.main_colors.is_empty());
        assert!(!java.main_colors.is_empty());
    }

    #[test]
    fn test_calculate_optimal_size_no_scaling() {
        assert_eq!(calculate_optimal_size(100, 100), (100, 100));
        assert_eq!(calculate_optimal_size(50, 50), (50, 50));
        assert_eq!(calculate_optimal_size(112, 112), (112, 112));
    }

    #[test]
    fn test_calculate_optimal_size_with_scaling() {
        assert_eq!(calculate_optimal_size(200, 200), (111, 111));
        assert_eq!(calculate_optimal_size(500, 100), (250, 50));
        assert_eq!(calculate_optimal_size(1000, 1000), (112, 112));
    }

    #[test]
    fn test_calculate_optimal_size_extreme_aspect_ratio() {
        let (w, h) = calculate_optimal_size(10000, 10);
        assert_eq!(w, 3541);
        assert_eq!(h, 3);
    }

    #[test]
    fn test_calculate_optimal_size_minimum_one_pixel() {
        let (w, h) = calculate_optimal_size(100000, 1);
        assert_eq!(w, 35417);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_wrap_degrees() {
        assert_eq!(wrap_degrees(0), 0);
        assert_eq!(wrap_degrees(360), 0);
        assert_eq!(wrap_degrees(450), 90);
        assert_eq!(wrap_degrees(-10), 350);
        assert_eq!(wrap_degrees(-370), 350);
        assert_eq!(wrap_degrees(720), 0);
        assert_eq!(wrap_degrees(-720), 0);
        assert_eq!(wrap_degrees(180), 180);
        assert_eq!(wrap_degrees(-180), 180);
    }

    #[test]
    fn test_dark_hints_bright_image() {
        let img = create_solid_color_image(Argb::new(255, 255, 255, 255), 10, 10);
        let hints = calculate_dark_hints(&img, 0.0);
        assert!(hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
        assert!(!hints.contains(ColorHints::SUPPORTS_DARK_THEME));
    }

    #[test]
    fn test_dark_hints_dark_image() {
        let img = create_solid_color_image(Argb::new(255, 0, 0, 0), 10, 10);
        let hints = calculate_dark_hints(&img, 0.0);
        assert!(!hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
        assert!(hints.contains(ColorHints::SUPPORTS_DARK_THEME));
    }

    #[test]
    fn test_dark_hints_mid_gray() {
        let img = create_solid_color_image(Argb::new(255, 128, 128, 128), 10, 10);
        let hints = calculate_dark_hints(&img, 0.0);
        assert!(!hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
        assert!(hints.contains(ColorHints::SUPPORTS_DARK_THEME));
    }

    #[test]
    fn test_dark_hints_with_dimming() {
        let img = create_solid_color_image(Argb::new(255, 255, 255, 255), 10, 10);
        let hints_no_dim = calculate_dark_hints(&img, 0.0);
        let hints_dim = calculate_dark_hints(&img, 0.5);
        assert!(
            !hints_dim.contains(ColorHints::SUPPORTS_DARK_TEXT)
                || hints_no_dim.contains(ColorHints::SUPPORTS_DARK_TEXT)
        );
    }

    #[test]
    fn test_dark_hints_with_dark_pixels() {
        let mut img = ImageBuffer::new(10, 10);
        for (x, _y, pixel) in img.enumerate_pixels_mut() {
            if x < 2 {
                *pixel = Rgba([0, 0, 0, 255]);
            } else {
                *pixel = Rgba([255, 255, 255, 255]);
            }
        }
        let img = DynamicImage::ImageRgba8(img);
        let hints = calculate_dark_hints(&img, 0.0);
        assert!(!hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
    }

    #[test]
    fn test_wallpaper_colors_from_color_map() {
        let mut map = HashMap::new();
        map.insert(0xffff0000, 100);
        map.insert(0xff00ff00, 50);
        map.insert(0xff0000ff, 25);

        let colors = WallpaperColors::from_color_map(map, ColorHints::empty());
        assert!(!colors.main_colors.is_empty());
        assert_eq!(colors.all_colors.len(), 3);
    }

    #[test]
    fn test_wallpaper_colors_from_color_map_java_strategy() {
        let mut map = HashMap::new();
        map.insert(0xffff0000, 100);
        map.insert(0xff00ff00, 50);
        map.insert(0xff0000ff, 25);

        let colors = WallpaperColors::from_color_map_with_strategy(
            map,
            ColorHints::empty(),
            ScoringStrategy::JavaOriginal,
        );
        assert!(!colors.main_colors.is_empty());
    }

    #[test]
    fn test_wallpaper_colors_accessors() {
        let img = create_test_image();
        let colors = WallpaperColors::from_bitmap(img);

        assert!(colors.primary_color().alpha > 0);
        let _ = colors.secondary_color();
        let _ = colors.tertiary_color();
        assert!(!colors.main_colors().is_empty());
        assert!(!colors.all_colors().is_empty());
    }

    #[test]
    fn test_wallpaper_colors_equality() {
        let img = create_test_image();
        let colors1 = WallpaperColors::from_bitmap(img.clone());
        let colors2 = WallpaperColors::from_bitmap(img);
        assert_eq!(colors1, colors2);
    }

    #[test]
    fn test_wallpaper_colors_clone() {
        let img = create_test_image();
        let colors = WallpaperColors::from_bitmap(img);
        let cloned = colors.clone();
        assert_eq!(colors, cloned);
    }

    #[test]
    fn test_large_image_scaling() {
        let img = create_gradient_image(1000, 1000);
        let colors = WallpaperColors::from_bitmap(img);
        assert!(!colors.main_colors.is_empty());
    }

    #[test]
    fn test_small_image() {
        let img = create_solid_color_image(Argb::new(255, 255, 0, 0), 2, 2);
        let colors = WallpaperColors::from_bitmap(img);
        assert!(!colors.main_colors.is_empty());
    }

    #[test]
    fn test_single_pixel_image() {
        let img = create_solid_color_image(Argb::new(255, 0, 255, 0), 1, 1);
        let colors = WallpaperColors::from_bitmap(img);
        assert_eq!(colors.main_colors.len(), 1);
    }

    #[test]
    fn test_argb_to_u32_roundtrip() {
        let argb = Argb::new(255, 128, 64, 32);
        let u32_val = argb_to_u32(argb);
        let roundtrip = u32_to_argb(u32_val);
        assert_eq!(argb, roundtrip);
    }

    #[test]
    fn test_argb_to_u32_known_values() {
        assert_eq!(argb_to_u32(Argb::new(255, 0, 0, 0)), 0xff000000);
        assert_eq!(argb_to_u32(Argb::new(255, 255, 255, 255)), 0xffffffff);
        assert_eq!(argb_to_u32(Argb::new(255, 255, 0, 0)), 0xffff0000);
        assert_eq!(argb_to_u32(Argb::new(0, 0, 0, 0)), 0x00000000);
    }

    #[test]
    fn test_color_hints_bitflags() {
        let mut hints = ColorHints::empty();
        assert!(!hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
        
        hints |= ColorHints::SUPPORTS_DARK_TEXT;
        assert!(hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
        
        hints |= ColorHints::SUPPORTS_DARK_THEME;
        assert!(hints.contains(ColorHints::SUPPORTS_DARK_THEME));
        
        hints &= !ColorHints::SUPPORTS_DARK_TEXT;
        assert!(!hints.contains(ColorHints::SUPPORTS_DARK_TEXT));
        assert!(hints.contains(ColorHints::SUPPORTS_DARK_THEME));
    }

    #[test]
    fn test_color_hints_combinations() {
        let all = ColorHints::SUPPORTS_DARK_TEXT | ColorHints::SUPPORTS_DARK_THEME | ColorHints::FROM_BITMAP;
        assert!(all.contains(ColorHints::SUPPORTS_DARK_TEXT));
        assert!(all.contains(ColorHints::SUPPORTS_DARK_THEME));
        assert!(all.contains(ColorHints::FROM_BITMAP));
    }

    #[test]
    fn test_scoring_strategy_equality() {
        assert_eq!(ScoringStrategy::MaterialColors, ScoringStrategy::MaterialColors);
        assert_eq!(ScoringStrategy::JavaOriginal, ScoringStrategy::JavaOriginal);
        assert_ne!(ScoringStrategy::MaterialColors, ScoringStrategy::JavaOriginal);
    }

    #[test]
    fn test_scoring_strategy_clone() {
        let strategy = ScoringStrategy::JavaOriginal;
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }

    #[test]
    fn test_hue_diff_same_hue() {
        let cam1: Cam16 = Argb::new(255, 255, 0, 0).into();
        let cam2: Cam16 = Argb::new(255, 255, 0, 0).into();
        assert!(hue_diff(&cam1, &cam2).abs() < 0.1);
    }

    #[test]
    fn test_hue_diff_opposite_hues() {
        let red: Cam16 = Argb::new(255, 255, 0, 0).into();
        let cyan: Cam16 = Argb::new(255, 0, 255, 255).into();
        let diff = hue_diff(&red, &cyan);
        assert!(diff > 150.0 && diff <= 180.0, "Expected hue diff around 180, got {}", diff);
    }

    #[test]
    fn test_score_cam_zero_proportion() {
        let cam: Cam16 = Argb::new(255, 255, 0, 0).into();
        let score = score_cam(&cam, 0.0);
        assert_eq!(score, cam.chroma);
    }

    #[test]
    fn test_score_cam_full_proportion() {
        let cam: Cam16 = Argb::new(255, 255, 0, 0).into();
        let score = score_cam(&cam, 1.0);
        assert_eq!(score, cam.chroma + 100.0);
    }

    #[test]
    fn test_hue_proportions_empty() {
        let cam_map: HashMap<u32, Cam16> = HashMap::new();
        let pop_map: HashMap<u32, u32> = HashMap::new();
        let props = hue_proportions_cam(&cam_map, &pop_map);
        assert!(props.iter().all(|&p| p == 0.0));
    }

    #[test]
    fn test_hue_proportions_single_color() {
        let red = Argb::new(255, 255, 0, 0);
        let mut cam_map = HashMap::new();
        let mut pop_map = HashMap::new();
        cam_map.insert(argb_to_u32(red), red.into());
        pop_map.insert(argb_to_u32(red), 100);
        
        let props = hue_proportions_cam(&cam_map, &pop_map);
        let total: f64 = props.iter().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_color_to_hue_proportion_single() {
        let red = Argb::new(255, 255, 0, 0);
        let mut cam_map = HashMap::new();
        cam_map.insert(argb_to_u32(red), red.into());
        
        let mut props = [0.0; 360];
        let cam: Cam16 = red.into();
        let hue = wrap_degrees(cam.hue.round() as i32);
        props[hue as usize] = 1.0;
        
        let result = color_to_hue_proportion_cam(&vec![argb_to_u32(red)], &cam_map, &props);
        assert_eq!(result.get(&argb_to_u32(red)), Some(&(1.0 + 0.0)));
    }

    #[test]
    fn test_from_bitmap_with_dim() {
        let img = create_test_image();
        let colors = WallpaperColors::from_bitmap_with_strategy(img, 0.5, ScoringStrategy::MaterialColors);
        assert!(!colors.main_colors.is_empty());
    }

    #[test]
    fn test_from_bitmap_both_strategies_on_gradient() {
        let img = create_gradient_image(100, 100);
        let material = WallpaperColors::from_bitmap(img.clone());
        let java = WallpaperColors::from_bitmap_java(img);
        assert!(!material.main_colors.is_empty());
        assert!(!java.main_colors.is_empty());
    }

    #[test]
    fn test_wallpaper_colors_debug() {
        let img = create_test_image();
        let colors = WallpaperColors::from_bitmap(img);
        let debug_str = format!("{:?}", colors);
        assert!(debug_str.contains("WallpaperColors"));
    }

    #[test]
    fn test_transparent_image() {
        let img = create_solid_color_image(Argb::new(0, 255, 0, 0), 10, 10);
        let colors = WallpaperColors::from_bitmap(img);
        assert!(!colors.main_colors.is_empty());
    }

    #[test]
    fn test_black_and_white_checkerboard() {
        let mut img = ImageBuffer::new(10, 10);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if (x + y) % 2 == 0 {
                *pixel = Rgba([255, 255, 255, 255]);
            } else {
                *pixel = Rgba([0, 0, 0, 255]);
            }
        }
        let img = DynamicImage::ImageRgba8(img);
        let colors = WallpaperColors::from_bitmap(img);
        assert!(!colors.main_colors.is_empty());
    }
}
