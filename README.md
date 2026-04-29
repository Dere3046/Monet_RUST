# Monet

Generate Material You themes from wallpaper images. Rust port of Android's wallpaper color extraction.

[中文版](README.zh.md)

## Usage

```bash
monet <wallpaper.png> [options]
  --strategy material|java   Scoring strategy (default: material)
  --output, -o <file>        Save JSON to file instead of stdout
```

## References

- [WallpaperColors.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/app/WallpaperColors.java)
- [WallpaperColors.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/app/WallpaperColors.java)

## License

Dual-licensed under MIT and Apache-2.0. See [LICENSE](LICENSE).
