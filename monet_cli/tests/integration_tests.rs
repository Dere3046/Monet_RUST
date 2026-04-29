use image::{ImageBuffer, Rgba};
use material_colors::color::Argb;
use material_colors::theme::ThemeBuilder;
use std::collections::HashMap;
use wallpaper_colors::WallpaperColors;

fn create_test_image() -> image::DynamicImage {
    let mut img = ImageBuffer::new(100, 100);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x < 50 && y < 50 {
            *pixel = Rgba([255, 0, 0, 255]);
        } else if x >= 50 && y < 50 {
            *pixel = Rgba([0, 255, 0, 255]);
        } else if x < 50 && y >= 50 {
            *pixel = Rgba([0, 0, 255, 255]);
        } else {
            *pixel = Rgba([255, 255, 0, 255]);
        }
    }
    image::DynamicImage::ImageRgba8(img)
}

fn create_solid_image(color: Argb) -> image::DynamicImage {
    let mut img = ImageBuffer::new(50, 50);
    for (_, _, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgba([color.red, color.green, color.blue, color.alpha]);
    }
    image::DynamicImage::ImageRgba8(img)
}

#[test]
fn test_end_to_end_material_strategy() {
    let img = create_test_image();
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    
    let theme = ThemeBuilder::with_source(seed).build();
    
    assert!(theme.schemes.light.primary.alpha > 0);
    assert!(theme.schemes.dark.primary.alpha > 0);
}

#[test]
fn test_end_to_end_java_strategy() {
    let img = create_test_image();
    let colors = WallpaperColors::from_bitmap_java(img);
    let seed = *colors.primary_color();
    
    let theme = ThemeBuilder::with_source(seed).build();
    
    assert!(theme.schemes.light.primary.alpha > 0);
    assert!(theme.schemes.dark.primary.alpha > 0);
}

#[test]
fn test_theme_builder_variants() {
    let img = create_test_image();
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    
    let theme_default = ThemeBuilder::with_source(seed).build();
    let theme_vibrant = ThemeBuilder::with_source(seed)
        .variant(material_colors::dynamic_color::Variant::Vibrant)
        .build();
    
    // Different variants should produce different themes
    assert_ne!(
        theme_default.schemes.light.primary,
        theme_vibrant.schemes.light.primary
    );
}

#[test]
fn test_json_export_structure() {
    let img = create_test_image();
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    let _seed_u32 = argb_to_u32(seed);
    
    let theme = ThemeBuilder::with_source(seed).build();
    
    let light: HashMap<String, String> = theme.schemes.light.into_iter().map(|(k, v)| (k, v.to_hex_with_pound())).collect();
    let dark: HashMap<String, String> = theme.schemes.dark.into_iter().map(|(k, v)| (k, v.to_hex_with_pound())).collect();
    
    // Verify key colors exist
    assert!(light.contains_key("primary"));
    assert!(light.contains_key("on_primary"));
    assert!(light.contains_key("background"));
    assert!(dark.contains_key("primary"));
    assert!(dark.contains_key("on_primary"));
    assert!(dark.contains_key("background"));
    
    // Verify hex format
    let primary_hex = light.get("primary").unwrap();
    assert!(primary_hex.starts_with('#'));
    assert_eq!(primary_hex.len(), 7);
}

#[test]
fn test_seed_extraction_red() {
    let img = create_solid_image(Argb::new(255, 255, 0, 0));
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    
    // Should extract red or a red-adjacent color
    assert!(seed.red > seed.blue);
    assert!(seed.red > seed.green);
}

#[test]
fn test_seed_extraction_blue() {
    let img = create_solid_image(Argb::new(255, 0, 0, 255));
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    
    // Should extract blue or a blue-adjacent color
    assert!(seed.blue > seed.red);
    assert!(seed.blue > seed.green);
}

#[test]
fn test_seed_extraction_green() {
    let img = create_solid_image(Argb::new(255, 0, 255, 0));
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    
    // Should extract green or a green-adjacent color
    assert!(seed.green > seed.red);
    assert!(seed.green > seed.blue);
}

#[test]
fn test_palette_generation() {
    let img = create_test_image();
    let colors = WallpaperColors::from_bitmap(img);
    let seed = *colors.primary_color();
    
    let theme = ThemeBuilder::with_source(seed).build();
    
    // Verify all palettes exist
    let _ = theme.palettes.primary.tone(40);
    let _ = theme.palettes.secondary.tone(40);
    let _ = theme.palettes.tertiary.tone(40);
    let _ = theme.palettes.neutral.tone(40);
    let _ = theme.palettes.neutral_variant.tone(40);
    let _ = theme.palettes.error.tone(40);
}

#[test]
fn test_color_hints_on_bright_image() {
    let img = create_solid_image(Argb::new(255, 255, 255, 255));
    let colors = WallpaperColors::from_bitmap(img);
    
    assert!(colors.color_hints().contains(wallpaper_colors::ColorHints::SUPPORTS_DARK_TEXT));
    assert!(!colors.color_hints().contains(wallpaper_colors::ColorHints::SUPPORTS_DARK_THEME));
}

#[test]
fn test_color_hints_on_dark_image() {
    let img = create_solid_image(Argb::new(255, 0, 0, 0));
    let colors = WallpaperColors::from_bitmap(img);
    
    assert!(!colors.color_hints().contains(wallpaper_colors::ColorHints::SUPPORTS_DARK_TEXT));
    assert!(colors.color_hints().contains(wallpaper_colors::ColorHints::SUPPORTS_DARK_THEME));
}

#[test]
fn test_strategy_difference() {
    let img = create_test_image();
    let material = WallpaperColors::from_bitmap(img.clone());
    let java = WallpaperColors::from_bitmap_java(img);
    
    // Both should produce valid results
    assert!(!material.main_colors().is_empty());
    assert!(!java.main_colors().is_empty());
    
    // Results might differ
    let _ = material.primary_color();
    let _ = java.primary_color();
}

#[test]
fn test_wallpaper_colors_map_population() {
    let img = create_test_image();
    let colors = WallpaperColors::from_bitmap(img);
    
    // Should have recorded all colors
    assert!(!colors.all_colors().is_empty());
    
    // Total population should equal pixel count
    let total_pop: u32 = colors.all_colors().values().sum();
    assert_eq!(total_pop, 100 * 100);
}

fn argb_to_u32(argb: Argb) -> u32 {
    ((argb.alpha as u32) << 24)
        | ((argb.red as u32) << 16)
        | ((argb.green as u32) << 8)
        | (argb.blue as u32)
}
