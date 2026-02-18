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

    const icons = {
      auto: '🔄',
      light: '☀️',
      dark: '🌙'
    };
    
    // Show the icon for the current theme
    themeIcon.textContent = icons[this.currentTheme];
    
    // Update tooltip
    const tooltips = {
      auto: 'Theme: Auto',
      light: 'Theme: Light',
      dark: 'Theme: Dark'
    };
    
    themeToggle.setAttribute('title', tooltips[this.currentTheme]);
    themeToggle.setAttribute('aria-label', `Current theme: ${this.currentTheme}`);
    
    // Update active state in dropdown
    this.updateDropdownActiveState();
  }

  updateDropdownActiveState() {
    const themeOptions = document.querySelectorAll('.theme-option');
    themeOptions.forEach(option => {
      const optionTheme = option.getAttribute('data-theme');
      if (optionTheme === this.currentTheme) {
        option.classList.add('active');
      } else {
        option.classList.remove('active');
      }
    });
    
    // Also update mobile theme buttons
    const mobileThemeOptions = document.querySelectorAll('.mobile-theme-option');
    mobileThemeOptions.forEach(option => {
      const optionTheme = option.getAttribute('data-theme');
      if (optionTheme === this.currentTheme) {
        option.classList.add('active');
      } else {
        option.classList.remove('active');
      }
    });
  }

  setupThemeToggle() {
    const themeToggle = document.getElementById('themeToggle');
    const themeOptions = document.querySelectorAll('.theme-option');
    
    if (!themeToggle) {
      console.warn('Theme toggle button not found');
      return;
    }

    // Handle clicks on individual theme options
    themeOptions.forEach(option => {
      option.addEventListener('click', (e) => {
        e.stopPropagation();
        const selectedTheme = option.getAttribute('data-theme');
        this.applyTheme(selectedTheme);
        this.updateThemeIcon();
        
        // Add a subtle animation effect
        option.style.transform = 'scale(0.95)';
        setTimeout(() => {
          option.style.transform = '';
        }, 150);
      });

      // Keyboard support for options
      option.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          option.click();
        }
      });
    });

    // Keyboard support for toggle button
    themeToggle.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        // Cycle through themes on keyboard activation
        const nextTheme = this.getNextTheme();
        this.applyTheme(nextTheme);
        this.updateThemeIcon();
      }
    });
    
    // Setup mobile theme buttons
    this.setupMobileThemeButtons();
  }
  
  setupMobileThemeButtons() {
    const mobileThemeOptions = document.querySelectorAll('.mobile-theme-option');
    
    mobileThemeOptions.forEach(option => {
      option.addEventListener('click', () => {
        const theme = option.getAttribute('data-theme');
        this.applyTheme(theme);
        this.updateThemeIcon();
      });
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
 * Design Style Management System
 * Handles switching between Classic and Glassmorphism designs
 */
class DesignManager {
  constructor() {
    this.designKey = 'lazyllama-design';
    this.designs = ['classic', 'glassmorphism'];
    this.currentDesign = this.getStoredDesign() || 'classic';
    
    this.init();
  }

  init() {
    // Set initial design
    this.applyDesign(this.currentDesign);
    
    // Setup design toggle button
    this.setupDesignToggle();
    
    // Update design toggle icon
    this.updateDesignIcon();
  }

  getStoredDesign() {
    try {
      return localStorage.getItem(this.designKey);
    } catch (e) {
      console.warn('LocalStorage not available, using default design');
      return null;
    }
  }

  storeDesign(design) {
    try {
      localStorage.setItem(this.designKey, design);
    } catch (e) {
      console.warn('Cannot store design preference');
    }
  }

  applyDesign(design) {
    document.documentElement.setAttribute('data-design', design);
    this.currentDesign = design;
    this.storeDesign(design);
  }

  toggleDesign() {
    const currentIndex = this.designs.indexOf(this.currentDesign);
    const nextIndex = (currentIndex + 1) % this.designs.length;
    return this.designs[nextIndex];
  }

  updateDesignIcon() {
    const designIcon = document.querySelector('.design-icon');
    const designToggle = document.querySelector('.design-toggle');
    
    if (!designIcon || !designToggle) return;

    const icons = {
      classic: '✨',
      glassmorphism: '🎨'
    };
    
    // Show the icon for what's currently active
    designIcon.textContent = icons[this.currentDesign];
    
    // Update tooltip
    const tooltips = {
      classic: 'Switch to Glassmorphism design',
      glassmorphism: 'Switch to Classic design'
    };
    
    designToggle.setAttribute('title', tooltips[this.currentDesign]);
    designToggle.setAttribute('aria-label', tooltips[this.currentDesign]);
  }

  setupDesignToggle() {
    const designToggle = document.getElementById('designToggle');
    
    if (!designToggle) {
      console.warn('Design toggle button not found');
      return;
    }

    designToggle.addEventListener('click', () => {
      const nextDesign = this.toggleDesign();
      this.applyDesign(nextDesign);
      this.updateDesignIcon();
      this.announceDesignChange();
      
      // Add a fun animation effect
      designToggle.style.transform = 'scale(0.9) rotate(180deg)';
      setTimeout(() => {
        designToggle.style.transform = '';
      }, 300);
    });

    // Keyboard support
    designToggle.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        designToggle.click();
      }
    });
    
    // Setup mobile menu design buttons
    this.setupMobileDesignButtons();
  }
  
  setupMobileDesignButtons() {
    const mobileDesignButtons = document.querySelectorAll('.mobile-menu-subitem[data-design]');
    
    mobileDesignButtons.forEach(button => {
      button.addEventListener('click', () => {
        const design = button.getAttribute('data-design');
        this.applyDesign(design);
        this.updateDesignIcon();
        this.updateMobileDesignButtons();
        this.announceDesignChange();
      });
    });
    
    // Set initial active state
    this.updateMobileDesignButtons();
  }
  
  updateMobileDesignButtons() {
    const mobileDesignButtons = document.querySelectorAll('.mobile-menu-subitem[data-design]');
    
    mobileDesignButtons.forEach(button => {
      const design = button.getAttribute('data-design');
      if (design === this.currentDesign) {
        button.classList.add('active');
      } else {
        button.classList.remove('active');
      }
    });
  }

  announceDesignChange() {
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
    
    const designName = this.currentDesign === 'glassmorphism' ? 'Glassmorphism' : 'Classic';
    announcement.textContent = `Design switched to ${designName}`;
    
    document.body.appendChild(announcement);
    
    setTimeout(() => {
      document.body.removeChild(announcement);
    }, 1000);
  }

  // Public method to manually set design
  setDesign(design) {
    if (this.designs.includes(design)) {
      this.applyDesign(design);
      this.updateDesignIcon();
    } else {
      console.warn(`Invalid design: ${design}. Available designs:`, this.designs);
    }
  }

  // Public method to get current design info
  getDesignInfo() {
    return {
      current: this.currentDesign,
      available: this.designs
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
    // Resources are now loaded locally from CSS
    // No external preloading needed
  }
}

/**
 * Mobile Menu Manager
 * Handles hamburger menu toggle for mobile devices
 */
class MobileMenu {
  constructor() {
    this.menuToggle = document.getElementById('mobileMenuToggle');
    this.navLinks = document.getElementById('navLinks');
    this.navLinksItems = document.querySelectorAll('.nav-link');
    this.isOpen = false;
    
    this.init();
  }

  init() {
    if (!this.menuToggle || !this.navLinks) {
      console.warn('Mobile menu elements not found');
      return;
    }

    // Toggle menu on button click
    this.menuToggle.addEventListener('click', () => this.toggleMenu());

    // Close menu when clicking nav links
    this.navLinksItems.forEach(link => {
      link.addEventListener('click', () => {
        if (this.isOpen) {
          this.closeMenu();
        }
      });
    });

    // Close menu when clicking outside
    document.addEventListener('click', (e) => {
      if (this.isOpen && 
          !this.navLinks.contains(e.target) && 
          !this.menuToggle.contains(e.target)) {
        this.closeMenu();
      }
    });

    // Close menu on escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.isOpen) {
        this.closeMenu();
      }
    });

    // Handle window resize
    window.addEventListener('resize', () => {
      if (window.innerWidth > 768 && this.isOpen) {
        this.closeMenu();
      }
    });
  }

  toggleMenu() {
    if (this.isOpen) {
      this.closeMenu();
    } else {
      this.openMenu();
    }
  }

  openMenu() {
    this.navLinks.classList.add('active');
    this.menuToggle.classList.add('active');
    this.menuToggle.setAttribute('aria-expanded', 'true');
    this.isOpen = true;
    
    // Prevent body scroll when menu is open
    document.body.style.overflow = 'hidden';
  }

  closeMenu() {
    this.navLinks.classList.remove('active');
    this.menuToggle.classList.remove('active');
    this.menuToggle.setAttribute('aria-expanded', 'false');
    this.isOpen = false;
    
    // Restore body scroll
    document.body.style.overflow = '';
  }
}

/**
 * Cookie Consent Manager
 * Handles GDPR cookie consent banner
 */
class CookieConsent {
  constructor() {
    this.cookieName = 'lazyllama_cookie_consent';
    this.cookieDuration = 365; // days
    this.banner = document.getElementById('cookieConsent');
    this.acceptBtn = document.getElementById('cookieAccept');
    this.init();
  }

  init() {
    if (!this.banner || !this.acceptBtn) return;

    if (this.hasConsented()) {
      this.hideBanner();
    } else {
      this.showBanner();
    }

    this.acceptBtn.addEventListener('click', () => this.acceptCookies());
    this.acceptBtn.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        this.acceptCookies();
      }
    });
  }

  hasConsented() {
    return this.getCookie(this.cookieName) === 'true';
  }

  acceptCookies() {
    this.setCookie(this.cookieName, 'true', this.cookieDuration);
    this.hideBanner();
  }

  showBanner() {
    if (this.banner) {
      this.banner.removeAttribute('hidden');
      void this.banner.offsetHeight; // Force reflow
      this.banner.style.display = 'block';
    }
  }

  hideBanner() {
    if (this.banner) {
      this.banner.style.animation = 'slideDownOut 0.3s ease-in';
      setTimeout(() => {
        this.banner.setAttribute('hidden', '');
        this.banner.style.display = 'none';
      }, 300);
    }
  }

  setCookie(name, value, days) {
    const date = new Date();
    date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
    const expires = `expires=${date.toUTCString()}`;
    document.cookie = `${name}=${value};${expires};path=/;SameSite=Strict`;
  }

  getCookie(name) {
    const nameEQ = name + "=";
    const ca = document.cookie.split(';');
    for (let i = 0; i < ca.length; i++) {
      let c = ca[i];
      while (c.charAt(0) === ' ') c = c.substring(1, c.length);
      if (c.indexOf(nameEQ) === 0) return c.substring(nameEQ.length, c.length);
    }
    return null;
  }
  
  deleteCookie(name) {
    // Helper function to manually clear the consent cookie for testing
    document.cookie = `${name}=;expires=Thu, 01 Jan 1970 00:00:00 UTC;path=/;SameSite=Strict`;
    console.log(`Cookie ${name} deleted`);
  }
}

/**
 * Initialize all components when DOM is ready
 */
document.addEventListener('DOMContentLoaded', () => {
  // Initialize theme management
  window.themeManager = new ThemeManager();
  
  // Initialize design management
  window.designManager = new DesignManager();
  
  // Initialize cookie consent
  window.cookieConsent = new CookieConsent();
  
  // Initialize mobile menu
  window.mobileMenu = new MobileMenu();
  
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
  module.exports = { ThemeManager, DesignManager, SmoothScroll, PerformanceEnhancements, CookieConsent, MobileMenu };
}