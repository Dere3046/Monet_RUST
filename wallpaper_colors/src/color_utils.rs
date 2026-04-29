use material_colors::color::Argb;

/// Convert color to HSL [hue, saturation, lightness].
pub fn color_to_hsl(argb: Argb) -> [f32; 3] {
    let r = argb.red as f32 / 255.0;
    let g = argb.green as f32 / 255.0;
    let b = argb.blue as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    let saturation = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    [hue, saturation, lightness]
}

/// Set the alpha component of a color.
pub fn set_alpha_component(argb: Argb, alpha: u8) -> Argb {
    Argb {
        alpha,
        red: argb.red,
        green: argb.green,
        blue: argb.blue,
    }
}

/// Composite two colors. Foreground over background.
pub fn composite_colors(foreground: Argb, background: Argb) -> Argb {
    let fg_alpha = foreground.alpha as f32 / 255.0;
    let bg_alpha = background.alpha as f32 / 255.0;

    let alpha = fg_alpha + bg_alpha * (1.0 - fg_alpha);

    if alpha == 0.0 {
        return Argb::new(0, 0, 0, 0);
    }

    let red = ((foreground.red as f32 * fg_alpha
        + background.red as f32 * bg_alpha * (1.0 - fg_alpha))
        / alpha) as u8;
    let green = ((foreground.green as f32 * fg_alpha
        + background.green as f32 * bg_alpha * (1.0 - fg_alpha))
        / alpha) as u8;
    let blue = ((foreground.blue as f32 * fg_alpha
        + background.blue as f32 * bg_alpha * (1.0 - fg_alpha))
        / alpha) as u8;

    Argb {
        alpha: (alpha * 255.0) as u8,
        red,
        green,
        blue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_hsl_red() {
        let red = Argb::new(255, 255, 0, 0);
        let hsl = color_to_hsl(red);
        assert!((hsl[0] - 0.0).abs() < 1.0 || (hsl[0] - 360.0).abs() < 1.0);
        assert!((hsl[1] - 1.0).abs() < 0.01);
        assert!((hsl[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_green() {
        let green = Argb::new(255, 0, 255, 0);
        let hsl = color_to_hsl(green);
        assert!((hsl[0] - 120.0).abs() < 1.0);
        assert!((hsl[1] - 1.0).abs() < 0.01);
        assert!((hsl[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_blue() {
        let blue = Argb::new(255, 0, 0, 255);
        let hsl = color_to_hsl(blue);
        assert!((hsl[0] - 240.0).abs() < 1.0);
        assert!((hsl[1] - 1.0).abs() < 0.01);
        assert!((hsl[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_white() {
        let white = Argb::new(255, 255, 255, 255);
        let hsl = color_to_hsl(white);
        assert_eq!(hsl[0], 0.0);
        assert_eq!(hsl[1], 0.0);
        assert!((hsl[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_black() {
        let black = Argb::new(255, 0, 0, 0);
        let hsl = color_to_hsl(black);
        assert_eq!(hsl[0], 0.0);
        assert_eq!(hsl[1], 0.0);
        assert!((hsl[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_gray() {
        let gray = Argb::new(255, 128, 128, 128);
        let hsl = color_to_hsl(gray);
        assert_eq!(hsl[0], 0.0);
        assert_eq!(hsl[1], 0.0);
        assert!((hsl[2] - 0.5).abs() < 0.02);
    }

    #[test]
    fn test_color_to_hsl_cyan() {
        let cyan = Argb::new(255, 0, 255, 255);
        let hsl = color_to_hsl(cyan);
        assert!((hsl[0] - 180.0).abs() < 1.0);
        assert!((hsl[1] - 1.0).abs() < 0.01);
        assert!((hsl[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_magenta() {
        let magenta = Argb::new(255, 255, 0, 255);
        let hsl = color_to_hsl(magenta);
        assert!((hsl[0] - 300.0).abs() < 1.0);
        assert!((hsl[1] - 1.0).abs() < 0.01);
        assert!((hsl[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl_yellow() {
        let yellow = Argb::new(255, 255, 255, 0);
        let hsl = color_to_hsl(yellow);
        assert!((hsl[0] - 60.0).abs() < 1.0);
        assert!((hsl[1] - 1.0).abs() < 0.01);
        assert!((hsl[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_alpha_component() {
        let color = Argb::new(255, 100, 150, 200);
        let new_color = set_alpha_component(color, 128);
        assert_eq!(new_color.alpha, 128);
        assert_eq!(new_color.red, 100);
        assert_eq!(new_color.green, 150);
        assert_eq!(new_color.blue, 200);
    }

    #[test]
    fn test_set_alpha_component_full_opaque() {
        let color = Argb::new(0, 100, 150, 200);
        let new_color = set_alpha_component(color, 255);
        assert_eq!(new_color.alpha, 255);
    }

    #[test]
    fn test_set_alpha_component_full_transparent() {
        let color = Argb::new(255, 100, 150, 200);
        let new_color = set_alpha_component(color, 0);
        assert_eq!(new_color.alpha, 0);
    }

    #[test]
    fn test_composite_colors_opaque_over_opaque() {
        let fg = Argb::new(255, 255, 0, 0);
        let bg = Argb::new(255, 0, 0, 255);
        let result = composite_colors(fg, bg);
        // Opaque red over opaque blue should be red
        assert_eq!(result, Argb::new(255, 255, 0, 0));
    }

    #[test]
    fn test_composite_colors_transparent_over_opaque() {
        let fg = Argb::new(0, 255, 0, 0);
        let bg = Argb::new(255, 0, 0, 255);
        let result = composite_colors(fg, bg);
        // Transparent over blue should be blue
        assert_eq!(result, bg);
    }

    #[test]
    fn test_composite_colors_half_transparent_over_opaque() {
        let fg = Argb::new(128, 255, 0, 0);
        let bg = Argb::new(255, 0, 0, 255);
        let result = composite_colors(fg, bg);
        // Half-transparent red over blue should be purple-ish
        assert!(result.red > 100);
        assert!(result.blue > 100);
        assert_eq!(result.alpha, 255);
    }

    #[test]
    fn test_composite_colors_both_transparent() {
        let fg = Argb::new(128, 255, 0, 0);
        let bg = Argb::new(128, 0, 0, 255);
        let result = composite_colors(fg, bg);
        assert!(result.alpha < 255);
        assert!(result.alpha > 0);
    }

    #[test]
    fn test_composite_colors_both_fully_transparent() {
        let fg = Argb::new(0, 255, 0, 0);
        let bg = Argb::new(0, 0, 0, 255);
        let result = composite_colors(fg, bg);
        assert_eq!(result, Argb::new(0, 0, 0, 0));
    }

    #[test]
    fn test_composite_colors_white_over_black() {
        let white = Argb::new(255, 255, 255, 255);
        let black = Argb::new(255, 0, 0, 0);
        let result = composite_colors(white, black);
        assert_eq!(result, white);
    }

    #[test]
    fn test_composite_colors_black_over_white() {
        let black = Argb::new(255, 0, 0, 0);
        let white = Argb::new(255, 255, 255, 255);
        let result = composite_colors(black, white);
        assert_eq!(result, black);
    }

    #[test]
    fn test_composite_colors_identity() {
        let color = Argb::new(128, 100, 150, 200);
        let transparent = Argb::new(0, 0, 0, 0);
        let result = composite_colors(color, transparent);
        assert_eq!(result.alpha, color.alpha);
        assert_eq!(result.red, color.red);
    }
}
