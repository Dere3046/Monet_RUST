use material_colors::color::Argb;

/// Calculate luminance using WCAG formula.
pub fn calculate_luminance(argb: Argb) -> f64 {
    let r = luminance_component(argb.red);
    let g = luminance_component(argb.green);
    let b = luminance_component(argb.blue);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn luminance_component(c: u8) -> f64 {
    let c_norm = c as f64 / 255.0;
    if c_norm <= 0.03928 {
        c_norm / 12.92
    } else {
        ((c_norm + 0.055) / 1.055).powf(2.4)
    }
}

/// Calculate contrast ratio between two colors.
pub fn calculate_contrast(foreground: Argb, background: Argb) -> f64 {
    let l1 = calculate_luminance(foreground);
    let l2 = calculate_luminance(background);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luminance_black() {
        let black = Argb::new(255, 0, 0, 0);
        let lum = calculate_luminance(black);
        assert!(lum.abs() < 0.001);
    }

    #[test]
    fn test_luminance_white() {
        let white = Argb::new(255, 255, 255, 255);
        let lum = calculate_luminance(white);
        assert!((lum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_luminance_red() {
        let red = Argb::new(255, 255, 0, 0);
        let lum = calculate_luminance(red);
        assert!(lum > 0.2);
        assert!(lum < 0.3);
    }

    #[test]
    fn test_luminance_green() {
        let green = Argb::new(255, 0, 255, 0);
        let lum = calculate_luminance(green);
        assert!(lum > 0.7);
        assert!(lum < 0.8);
    }

    #[test]
    fn test_luminance_blue() {
        let blue = Argb::new(255, 0, 0, 255);
        let lum = calculate_luminance(blue);
        assert!(lum > 0.05);
        assert!(lum < 0.1);
    }

    #[test]
    fn test_luminance_gray_50() {
        let gray = Argb::new(255, 128, 128, 128);
        let lum = calculate_luminance(gray);
        assert!(lum > 0.2);
        assert!(lum < 0.3);
    }

    #[test]
    fn test_luminance_gray_128() {
        let gray = Argb::new(255, 128, 128, 128);
        let lum = calculate_luminance(gray);
        assert!(lum > 0.2);
        assert!(lum < 0.3);
    }

    #[test]
    fn test_luminance_dark_gray() {
        let gray = Argb::new(255, 64, 64, 64);
        let lum = calculate_luminance(gray);
        assert!(lum > 0.05);
        assert!(lum < 0.1);
    }

    #[test]
    fn test_luminance_light_gray() {
        let gray = Argb::new(255, 192, 192, 192);
        let lum = calculate_luminance(gray);
        assert!(lum > 0.5);
        assert!(lum < 0.6);
    }

    #[test]
    fn test_contrast_black_white() {
        let black = Argb::new(255, 0, 0, 0);
        let white = Argb::new(255, 255, 255, 255);
        let contrast = calculate_contrast(black, white);
        assert!((contrast - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_white_black() {
        let white = Argb::new(255, 255, 255, 255);
        let black = Argb::new(255, 0, 0, 0);
        let contrast = calculate_contrast(white, black);
        assert!((contrast - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_same_color() {
        let red = Argb::new(255, 255, 0, 0);
        let contrast = calculate_contrast(red, red);
        assert!((contrast - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_contrast_red_blue() {
        let red = Argb::new(255, 255, 0, 0);
        let blue = Argb::new(255, 0, 0, 255);
        let contrast = calculate_contrast(red, blue);
        assert!(contrast > 1.0);
        assert!(contrast < 5.0);
    }

    #[test]
    fn test_contrast_red_green() {
        let red = Argb::new(255, 255, 0, 0);
        let green = Argb::new(255, 0, 255, 0);
        let contrast = calculate_contrast(red, green);
        assert!(contrast > 1.0);
        assert!(contrast < 3.0);
    }

    #[test]
    fn test_contrast_with_alpha() {
        let transparent = Argb::new(128, 255, 0, 0);
        let opaque = Argb::new(255, 0, 0, 255);
        // Alpha is ignored in luminance calculation
        let contrast = calculate_contrast(transparent, opaque);
        assert!(contrast > 1.0);
    }

    #[test]
    fn test_contrast_wcag_aa_threshold() {
        // WCAG AA requires 4.5:1 for normal text
        let dark_gray = Argb::new(255, 64, 64, 64);
        let white = Argb::new(255, 255, 255, 255);
        let contrast = calculate_contrast(dark_gray, white);
        assert!(contrast > 4.5, "Contrast should meet WCAG AA: {}", contrast);
    }

    #[test]
    fn test_contrast_wcag_aaa_threshold() {
        // WCAG AAA requires 7:1 for normal text
        let black = Argb::new(255, 0, 0, 0);
        let white = Argb::new(255, 255, 255, 255);
        let contrast = calculate_contrast(black, white);
        assert!(contrast > 7.0, "Contrast should meet WCAG AAA: {}", contrast);
    }
}
