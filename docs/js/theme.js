/**
 * Theme Management System
 * Handles automatic theme detection and manual theme switching
 */

class ThemeManager {
  constructor() {
    this.themeKey = 'lazyllama-theme';
    this.themes = ['auto', 'light', 'dark'];
    this.currentTheme = this.getStoredTheme() || 'auto';
    
    this.init();
  }

  init() {
    // Set initial theme
    this.applyTheme(this.currentTheme);
    
    // Setup theme toggle button
    this.setupThemeToggle();
    
    // Listen for system theme changes when in auto mode
    this.setupSystemThemeListener();
    
    // Update theme toggle icon
    this.updateThemeIcon();
  }

  getStoredTheme() {
    try {
      return localStorage.getItem(this.themeKey);
    } catch (e) {
      console.warn('LocalStorage not available, using default theme');
      return null;
    }
  }

  storeTheme(theme) {
    try {
      localStorage.setItem(this.themeKey, theme);
    } catch (e) {
      console.warn('Cannot store theme preference');
    }
  }

  applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    this.currentTheme = theme;
    this.storeTheme(theme);
  }

  getNextTheme() {
    const currentIndex = this.themes.indexOf(this.currentTheme);
    const nextIndex = (currentIndex + 1) % this.themes.length;
    return this.themes[nextIndex];
  }

  getSystemTheme() {
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'light';
  }

  getEffectiveTheme() {
    if (this.currentTheme === 'auto') {
      return this.getSystemTheme();
    }
    return this.currentTheme;
  }

  updateThemeIcon() {
    const themeIcon = document.querySelector('.theme-icon');
    const themeToggle = document.querySelector('.theme-toggle');
    
    if (!themeIcon || !themeToggle) return;

    const effectiveTheme = this.getEffectiveTheme();
    const icons = {
      light: '🌙',
      dark: '☀️'
    };
    
    // Show the icon for the opposite theme (what clicking will activate)
    const iconToShow = effectiveTheme === 'dark' ? icons.dark : icons.light;
    themeIcon.textContent = iconToShow;
    
    // Update tooltip
    const tooltips = {
      auto: 'Switch to light theme',
      light: 'Switch to dark theme',
      dark: 'Switch to auto theme'
    };
    
    themeToggle.setAttribute('title', tooltips[this.currentTheme]);
    themeToggle.setAttribute('aria-label', tooltips[this.currentTheme]);
  }

  setupThemeToggle() {
    const themeToggle = document.getElementById('themeToggle');
    
    if (!themeToggle) {
      console.warn('Theme toggle button not found');
      return;
    }

    themeToggle.addEventListener('click', () => {
      const nextTheme = this.getNextTheme();
      this.applyTheme(nextTheme);
      this.updateThemeIcon();
      
      // Add a subtle animation effect
      themeToggle.style.transform = 'scale(0.95)';
      setTimeout(() => {
        themeToggle.style.transform = '';
      }, 150);
    });

    // Keyboard support
    themeToggle.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        themeToggle.click();
      }
    });
  }

  setupSystemThemeListener() {
    if (typeof window === 'undefined' || !window.matchMedia) return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleSystemThemeChange = (e) => {
      // Only update if currently in auto mode
      if (this.currentTheme === 'auto') {
        this.updateThemeIcon();
        this.announceThemeChange();
      }
    };

    // Modern browsers
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleSystemThemeChange);
    } 
    // Legacy support
    else if (mediaQuery.addListener) {
      mediaQuery.addListener(handleSystemThemeChange);
    }
  }

  announceThemeChange() {
    // Create a temporary announcement for screen readers
    const announcement = document.createElement('div');
    announcement.setAttribute('aria-live', 'polite');
    announcement.setAttribute('aria-atomic', 'true');
    announcement.className = 'sr-only';
    announcement.style.cssText = `
      position: absolute;
      width: 1px;
      height: 1px;
      padding: 0;
      margin: -1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
      white-space: nowrap;
      border: 0;
    `;
    
    const effectiveTheme = this.getEffectiveTheme();
    announcement.textContent = `Theme switched to ${effectiveTheme} mode`;
    
    document.body.appendChild(announcement);
    
    setTimeout(() => {
      document.body.removeChild(announcement);
    }, 1000);
  }

  // Public method to manually set theme (useful for testing or external control)
  setTheme(theme) {
    if (this.themes.includes(theme)) {
      this.applyTheme(theme);
      this.updateThemeIcon();
    } else {
      console.warn(`Invalid theme: ${theme}. Available themes:`, this.themes);
    }
  }

  // Public method to get current theme info
  getThemeInfo() {
    return {
      current: this.currentTheme,
      effective: this.getEffectiveTheme(),
      available: this.themes,
      system: this.getSystemTheme()
    };
  }
}

/**
 * Smooth Scroll Enhancement
 * Adds smooth scrolling with offset for fixed navigation
 */
class SmoothScroll {
  constructor() {
    this.offset = 80; // Account for fixed navigation
    this.init();
  }

  init() {
    // Handle navigation links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
      anchor.addEventListener('click', (e) => {
        e.preventDefault();
        const targetId = anchor.getAttribute('href').substring(1);
        const targetElement = document.getElementById(targetId);
        
        if (targetElement) {
          this.scrollToElement(targetElement);
        }
      });
    });
  }

  scrollToElement(element) {
    const elementPosition = element.getBoundingClientRect().top;
    const offsetPosition = elementPosition + window.pageYOffset - this.offset;

    window.scrollTo({
      top: offsetPosition,
      behavior: 'smooth'
    });
  }
}

/**
 * Performance and Animation Enhancements
 */
class PerformanceEnhancements {
  constructor() {
    this.init();
  }

  init() {
    // Lazy loading for images (when added later)
    this.setupLazyLoading();
    
    // Intersection Observer for animations
    this.setupScrollAnimations();
    
    // Preload critical resources
    this.preloadResources();
  }

  setupLazyLoading() {
    // Placeholder for future image lazy loading
    if ('IntersectionObserver' in window) {
      const images = document.querySelectorAll('img[data-src]');
      const imageObserver = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            const img = entry.target;
            img.src = img.dataset.src;
            img.removeAttribute('data-src');
            imageObserver.unobserve(img);
          }
        });
      });

      images.forEach(img => imageObserver.observe(img));
    }
  }

  setupScrollAnimations() {
    if ('IntersectionObserver' in window) {
      const animatedElements = document.querySelectorAll('.feature-card, .link-card');
      
      const animationObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            entry.target.style.opacity = '1';
            entry.target.style.transform = 'translateY(0)';
          }
        });
      }, {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
      });

      animatedElements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        animationObserver.observe(el);
      });
    }
  }

  preloadResources() {
    // Preload fonts
    const fontLinks = [
      'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap'
    ];

    fontLinks.forEach(href => {
      const link = document.createElement('link');
      link.rel = 'preload';
      link.as = 'style';
      link.href = href;
      document.head.appendChild(link);
    });
  }
}

/**
 * Initialize all components when DOM is ready
 */
document.addEventListener('DOMContentLoaded', () => {
  // Initialize theme management
  window.themeManager = new ThemeManager();
  
  // Initialize smooth scrolling
  new SmoothScroll();
  
  // Initialize performance enhancements
  new PerformanceEnhancements();
  
  // Add loading completion class for any CSS animations
  document.body.classList.add('loaded');
});

// Handle page visibility changes for performance
document.addEventListener('visibilitychange', () => {
  if (document.hidden) {
    // Page is now hidden - could pause animations or reduce activity
    document.body.classList.add('page-hidden');
  } else {
    // Page is now visible - resume full activity
    document.body.classList.remove('page-hidden');
  }
});

// Export for potential external use
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { ThemeManager, SmoothScroll, PerformanceEnhancements };
}