# LazyLlama Design System

## Overview

The LazyLlama website now features a dual-design system that allows users to switch between two distinct visual styles:

1. **Classic Design** - The original clean, modern design
2. **Glassmorphism Design** - A futuristic design with blue/turquoise/purple colors and sleek animations

Both designs support **light and dark themes** that can be independently controlled.

## Features

### Design Styles

#### Classic Design
- Clean and professional appearance
- Minimalist approach with subtle shadows
- Traditional layout and styling
- Optimized for readability

#### Glassmorphism Design
- Futuristic, modern aesthetic
- Blue/Turquoise/Purple color scheme
- Glass-like transparency effects with blur
- Animated background elements
- Smooth transitions and hover effects
- Enhanced visual depth with layered elements

### Theme Support

Both designs support three theme modes:

- **Auto**: Follows system preference (light/dark)
- **Light**: Always uses light theme
- **Dark**: Always uses dark theme

## Usage

### Switching Designs

Click the **✨ sparkle icon** in the navigation bar to toggle between Classic and Glassmorphism designs.

- In Classic mode: Shows ✨ (sparkle)
- In Glassmorphism mode: Shows 🎨 (palette)

### Switching Themes

Click the **🌙/☀️ icon** in the navigation bar to cycle through theme modes:

1. Auto → Light → Dark → Auto (repeats)

### Keyboard Accessibility

Both toggle buttons support keyboard navigation:
- **Tab**: Navigate to button
- **Enter** or **Space**: Toggle setting

## Technical Details

### File Structure

```
docs/
├── css/
│   ├── styles.css           # Base styles for Classic design
│   ├── glassmorphism.css    # Glassmorphism design overrides
│   └── fonts.css            # Font definitions
├── js/
│   └── theme.js             # Theme & Design management
└── index.html               # Main HTML file
```

### CSS Architecture

The design system uses **CSS Custom Properties** and **data attributes** for clean separation:

- `data-theme="auto|light|dark"` - Controls color theme
- `data-design="classic|glassmorphism"` - Controls design style

### Glassmorphism Features

#### Color Palette

**Light Theme:**
- Primary: Blue (#3b82f6)
- Secondary: Turquoise (#14b8a6)
- Accent: Purple (#a855f7)

**Dark Theme:**
- Darker backgrounds with enhanced contrast
- Glowing purple borders
- Deeper shadows

#### Animations

- **Float**: Subtle floating motion for terminal preview
- **Glow**: Pulsing glow effects
- **Slide-in**: Staggered entrance animations
- **Shimmer**: Animated gradient text
- **Pulse**: Breathing cursor effect
- **Rotate**: Rotating background elements

#### Glass Effects

- Backdrop blur filters (8px, 16px, 24px)
- Semi-transparent overlays
- Layered depth with shadows
- Smooth color transitions

### Browser Support

The design system supports all modern browsers:
- Chrome/Edge (latest 2 versions)
- Firefox (latest 2 versions)
- Safari (latest 2 versions)

Graceful degradation for older browsers:
- Falls back to solid colors if backdrop-filter is unsupported
- Animations respect `prefers-reduced-motion` media query

## Accessibility Features

- **Keyboard Navigation**: Full keyboard support for all toggles
- **Screen Reader Announcements**: Design/theme changes are announced
- **Reduced Motion**: Respects user's motion preferences
- **Color Contrast**: Both designs meet WCAG standards
- **Focus Indicators**: Clear focus states for all interactive elements

## Performance

### Optimizations

- CSS transforms and opacity for GPU-accelerated animations
- Lazy-loaded animations with Intersection Observer
- Minimal reflows and repaints
- LocalStorage for instant preference recall

### File Sizes

- `glassmorphism.css`: ~15 KB (uncompressed)
- No additional JavaScript libraries required
- Leverages existing theme management system

## Customization

To modify the Glassmorphism color scheme, edit the CSS custom properties in `glassmorphism.css`:

```css
[data-design="glassmorphism"] {
  --glass-color-primary: #3b82f6;    /* Blue */
  --glass-color-secondary: #14b8a6;  /* Turquoise */
  --glass-color-accent: #a855f7;     /* Purple */
  /* ... */
}
```

## Future Enhancements

Potential improvements:
- Additional design styles (e.g., Minimalist, Cyberpunk)
- Per-section design overrides
- Color theme customization
- Animation speed controls
- Preset combinations (Design + Theme)

## Credits

- **Design**: Inspired by modern glassmorphism trends
- **Animations**: Custom CSS keyframe animations
- **Implementation**: JavaScript class-based architecture
- **Fonts**: Inter & JetBrains Mono (OFL-1.1)

---

For questions or suggestions, please open an issue on [GitHub](https://github.com/Pommersche92/lazyllama).
