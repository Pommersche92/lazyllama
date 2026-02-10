# LazyLlama GitHub Pages

This directory contains the static website for LazyLlama, hosted on GitHub Pages.

## 🌐 Live Site

The website is available at: [https://pommersche92.github.io/lazyllama](https://pommersche92.github.io/lazyllama)

## 📁 Structure

```
docs/
├── index.html          # Main landing page
├── css/
│   └── styles.css      # Comprehensive styling with light/dark theme support
├── js/
│   └── theme.js        # Theme management and interactive features
├── .nojekyll          # Tells GitHub Pages to serve static files directly
├── _config.yml        # GitHub Pages configuration (optional)
└── README.md          # This file
```

## ✨ Features

- **Modern Design**: Clean, professional layout with smooth animations
- **Light/Dark Theme**: Automatic system preference detection with manual toggle
- **Responsive**: Optimized for desktop, tablet, and mobile devices
- **Accessible**: WCAG compliant with proper ARIA labels and keyboard navigation
- **Fast**: Optimized for performance with efficient CSS and minimal JavaScript
- **SEO Optimized**: Meta tags, Open Graph, and Twitter Card support

## 🎨 Design Elements

- **Hero Section**: Eye-catching introduction with terminal preview
- **Features Grid**: Showcase of LazyLlama's key capabilities
- **Installation Guide**: Step-by-step setup instructions
- **Keyboard Controls**: Visual reference for all shortcuts
- **Resource Links**: Direct links to GitHub, Crates.io, documentation, and developer website

## 🔧 Theme System

The website includes a sophisticated theme management system:

- **Auto Mode**: Follows system preference (default)
- **Light Mode**: Traditional light theme
- **Dark Mode**: Modern dark theme
- **Smooth Transitions**: Seamless switching between themes
- **Persistent**: Theme preference saved in localStorage

## 🚀 Deployment

The site is automatically deployed via GitHub Pages when changes are pushed to the `main` branch. GitHub Pages serves the files from the `/docs` directory.

### Local Development

To preview locally, you can use any static file server:

```bash
# Using Python 3
cd docs
python -m http.server 8000

# Using Node.js
npx serve docs

# Using PHP
cd docs
php -S localhost:8000
```

Then open [http://localhost:8000](http://localhost:8000) in your browser.

## 📝 Customization

### Colors

Theme colors are defined as CSS custom properties in `css/styles.css`. You can modify the color scheme by updating the `:root` and `[data-theme="dark"]` selectors.

### Content

The main content is in `index.html`. Update sections as needed while maintaining the responsive grid layouts and accessibility features.

### Functionality

Additional JavaScript functionality can be added to `js/theme.js` or new JS files can be created and included in the HTML.

## 🔗 Links Configuration

The website includes links to:

- GitHub Repository: `https://github.com/Pommersche92/lazyllama`
- Crates.io Package: `https://crates.io/crates/lazyllama`
- API Documentation: `https://docs.rs/lazyllama`
- Developer Website: `https://geisel-web.de/`

Update these URLs in `index.html` if they change.

## 🌍 Browser Support

The website supports all modern browsers:

- Chrome/Chromium 60+
- Firefox 55+
- Safari 12+
- Edge 79+

Progressive enhancement ensures basic functionality on older browsers.

## 📱 Responsive Breakpoints

- Desktop: 1200px+
- Tablet: 768px - 1199px
- Mobile: < 768px
- Small Mobile: < 480px

The design adapts gracefully across all screen sizes.
