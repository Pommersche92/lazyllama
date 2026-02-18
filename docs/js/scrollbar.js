/**
 * Custom Scrollbar Implementation
 * Performance-optimized with requestAnimationFrame and passive listeners
 */

class CustomScrollbar {
  constructor() {
    this.scrollbar = null;
    this.thumb = null;
    this.isDragging = false;
    this.startY = 0;
    this.startScrollTop = 0;
    this.rafId = null;
    this.lastScrollTop = 0;
    
    this.init();
  }
  
  init() {
    // Create scrollbar elements
    this.createScrollbarElements();
    
    // Bind events with performance optimizations
    this.bindEvents();
    
    // Initial update
    this.updateScrollbar();
  }
  
  createScrollbarElements() {
    // Create scrollbar container
    this.scrollbar = document.createElement('div');
    this.scrollbar.className = 'custom-scrollbar';
    
    // Create scrollbar thumb
    this.thumb = document.createElement('div');
    this.thumb.className = 'custom-scrollbar-thumb';
    
    this.scrollbar.appendChild(this.thumb);
    document.body.appendChild(this.scrollbar);
  }
  
  bindEvents() {
    // Mouse events for thumb dragging
    this.thumb.addEventListener('mousedown', this.onThumbMouseDown.bind(this), { passive: false });
    
    // Scroll event with passive listener for better performance
    window.addEventListener('scroll', this.onScroll.bind(this), { passive: true });
    
    // Resize event to recalculate scrollbar
    window.addEventListener('resize', this.onResize.bind(this), { passive: true });
    
    // Touch events for mobile
    this.thumb.addEventListener('touchstart', this.onThumbTouchStart.bind(this), { passive: false });
    
    // Click on track to jump
    this.scrollbar.addEventListener('click', this.onTrackClick.bind(this));
  }
  
  onScroll() {
    // Use requestAnimationFrame to throttle updates
    if (!this.rafId) {
      this.rafId = requestAnimationFrame(() => {
        this.updateScrollbar();
        this.rafId = null;
      });
    }
  }
  
  onResize() {
    // Debounced resize handler
    clearTimeout(this.resizeTimeout);
    this.resizeTimeout = setTimeout(() => {
      this.updateScrollbar();
    }, 150);
  }
  
  updateScrollbar() {
    const scrollHeight = document.documentElement.scrollHeight;
    const clientHeight = document.documentElement.clientHeight;
    
    // Hide scrollbar if content fits
    if (scrollHeight <= clientHeight) {
      this.scrollbar.style.opacity = '0';
      this.scrollbar.style.pointerEvents = 'none';
      return;
    }
    
    this.scrollbar.style.opacity = '1';
    this.scrollbar.style.pointerEvents = 'auto';
    
    // Calculate thumb height (proportional to content)
    const thumbHeight = Math.max(
      (clientHeight / scrollHeight) * clientHeight,
      30 // Minimum thumb height
    );
    
    // Calculate thumb position
    const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const maxScroll = scrollHeight - clientHeight;
    const scrollPercentage = scrollTop / maxScroll;
    const maxThumbTop = clientHeight - thumbHeight;
    const thumbTop = scrollPercentage * maxThumbTop;
    
    // Update thumb styles (batched for performance)
    this.thumb.style.height = `${thumbHeight}px`;
    this.thumb.style.transform = `translateY(${thumbTop}px)`;
    
    // Show scrollbar on scroll, hide after delay
    this.showScrollbar();
  }
  
  showScrollbar() {
    this.scrollbar.classList.add('visible');
    
    // Hide after 1.5 seconds of inactivity
    clearTimeout(this.hideTimeout);
    this.hideTimeout = setTimeout(() => {
      if (!this.isDragging) {
        this.scrollbar.classList.remove('visible');
      }
    }, 1500);
  }
  
  onThumbMouseDown(e) {
    e.preventDefault();
    this.isDragging = true;
    this.startY = e.clientY;
    this.startScrollTop = window.pageYOffset || document.documentElement.scrollTop;
    
    document.addEventListener('mousemove', this.onThumbMouseMove.bind(this));
    document.addEventListener('mouseup', this.onThumbMouseUp.bind(this));
    
    this.scrollbar.classList.add('dragging');
    this.thumb.classList.add('active');
  }
  
  onThumbMouseMove(e) {
    if (!this.isDragging) return;
    
    const deltaY = e.clientY - this.startY;
    const clientHeight = document.documentElement.clientHeight;
    const scrollHeight = document.documentElement.scrollHeight;
    const thumbHeight = parseFloat(this.thumb.style.height);
    const maxThumbTop = clientHeight - thumbHeight;
    
    // Calculate scroll position
    const scrollPercentage = deltaY / maxThumbTop;
    const maxScroll = scrollHeight - clientHeight;
    const newScrollTop = this.startScrollTop + (scrollPercentage * maxScroll);
    
    // Scroll to new position
    window.scrollTo(0, Math.max(0, Math.min(newScrollTop, maxScroll)));
  }
  
  onThumbMouseUp() {
    this.isDragging = false;
    document.removeEventListener('mousemove', this.onThumbMouseMove.bind(this));
    document.removeEventListener('mouseup', this.onThumbMouseUp.bind(this));
    
    this.scrollbar.classList.remove('dragging');
    this.thumb.classList.remove('active');
  }
  
  onThumbTouchStart(e) {
    e.preventDefault();
    const touch = e.touches[0];
    this.isDragging = true;
    this.startY = touch.clientY;
    this.startScrollTop = window.pageYOffset || document.documentElement.scrollTop;
    
    const onTouchMove = (e) => {
      if (!this.isDragging) return;
      
      const touch = e.touches[0];
      const deltaY = touch.clientY - this.startY;
      const clientHeight = document.documentElement.clientHeight;
      const scrollHeight = document.documentElement.scrollHeight;
      const thumbHeight = parseFloat(this.thumb.style.height);
      const maxThumbTop = clientHeight - thumbHeight;
      
      const scrollPercentage = deltaY / maxThumbTop;
      const maxScroll = scrollHeight - clientHeight;
      const newScrollTop = this.startScrollTop + (scrollPercentage * maxScroll);
      
      window.scrollTo(0, Math.max(0, Math.min(newScrollTop, maxScroll)));
    };
    
    const onTouchEnd = () => {
      this.isDragging = false;
      document.removeEventListener('touchmove', onTouchMove);
      document.removeEventListener('touchend', onTouchEnd);
      
      this.scrollbar.classList.remove('dragging');
      this.thumb.classList.remove('active');
    };
    
    document.addEventListener('touchmove', onTouchMove, { passive: false });
    document.addEventListener('touchend', onTouchEnd, { passive: true });
    
    this.scrollbar.classList.add('dragging');
    this.thumb.classList.add('active');
  }
  
  onTrackClick(e) {
    // Don't scroll if clicking on thumb
    if (e.target === this.thumb) return;
    
    const rect = this.scrollbar.getBoundingClientRect();
    const clickY = e.clientY - rect.top;
    const clientHeight = document.documentElement.clientHeight;
    const scrollHeight = document.documentElement.scrollHeight;
    
    // Calculate target scroll position
    const scrollPercentage = clickY / clientHeight;
    const maxScroll = scrollHeight - clientHeight;
    const targetScroll = scrollPercentage * maxScroll;
    
    // Smooth scroll to position
    window.scrollTo({
      top: targetScroll,
      behavior: 'smooth'
    });
  }
  
  destroy() {
    // Cleanup
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
    }
    clearTimeout(this.hideTimeout);
    clearTimeout(this.resizeTimeout);
    
    if (this.scrollbar && this.scrollbar.parentNode) {
      this.scrollbar.parentNode.removeChild(this.scrollbar);
    }
  }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.customScrollbar = new CustomScrollbar();
  });
} else {
  window.customScrollbar = new CustomScrollbar();
}
