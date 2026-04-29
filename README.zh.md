# Monet

从壁纸图像生成 Material You 主题。Android 壁纸颜色提取的 Rust 移植版本。

[English](README.md)

## 用法

```bash
monet <wallpaper.png> [options]
  --strategy material|java   评分策略（默认：material）
  --output, -o <file>        保存 JSON 到文件而非标准输出
```

## 参考

- [WallpaperColors.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/app/WallpaperColors.java)
- [WallpaperColors.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/app/WallpaperColors.java)

## 许可证

MIT 和 Apache-2.0 双许可。详见 [LICENSE](LICENSE)。
