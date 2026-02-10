# LazyLlama Website Images

This directory contains images and graphics for the LazyLlama website.

## 📁 Recommended Structure

```
images/
├── logo.png              # LazyLlama logo (for favicon/meta tags)
├── hero-screenshot.png    # Terminal screenshot for hero section
├── preview.png           # Social media preview image (1200x630px)
├── features/             # Feature-specific illustrations
│   ├── streaming.png
│   ├── markdown.png
│   ├── scrolling.png
│   └── models.png
└── icons/                # Various icons if needed
    ├── github.svg
    ├── crates.svg
    └── docs.svg
```

## 🎨 Recommended Specifications

### Logo

- **Format**: PNG, SVG preferred
- **Size**: 200x200px minimum (square format)
- **Background**: Transparent
- **Style**: Should work on both light and dark backgrounds

### Hero Screenshot

- **Format**: PNG
- **Size**: 800x500px recommended
- **Content**: Terminal window showing LazyLlama in action
- **Style**: Should match the terminal preview in the hero section

### Social Media Preview

- **Format**: PNG or JPG
- **Size**: 1200x630px (Facebook/Twitter recommended)
- **Content**: LazyLlama logo + tagline
- **Style**: Eye-catching, representative of the project

### Feature Icons

- **Format**: SVG preferred, PNG fallback
- **Size**: 64x64px or scalable
- **Style**: Consistent with the emoji currently used
- **Colors**: Should work with both light and dark themes

## 🔧 Implementation

Once images are added to this directory, update the following in `index.html`:

1. **Favicon**: Replace the emoji SVG with actual logo
2. **Open Graph/Twitter images**: Update the `content` attribute in meta tags
3. **Hero section**: Replace terminal preview placeholder with actual screenshot
4. **Feature icons**: Replace emoji with actual icons

## 📝 Usage Notes

- All images should be optimized for web (compressed but high quality)
- Consider providing 2x versions for high-DPI displays
- SVG is preferred for icons as it scales perfectly
- Use WebP format for modern browser support with PNG/JPG fallbacks

## 🎯 Current Placeholders

The website currently uses:

- 🦙 emoji as favicon and branding
- CSS-generated terminal preview in hero section  
- Emoji icons (⚡, 🎨, etc.) for features
- No social media preview image

Replace these with actual graphics when available
