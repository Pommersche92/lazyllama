#!/usr/bin/env bash
#
# AppImageHub Submission Helper for LazyLlama
# Prepares files and provides instructions for AppImageHub submission
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"
APPIMAGEHUB_DIR="${PROJECT_ROOT}/appimagehub"

log_info() {
    echo -e "${BLUE}ℹ${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}✓${NC} $1" >&2
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1" >&2
}

log_step() {
    echo -e "${CYAN}${BOLD}▶ $1${NC}" >&2
}

# Get version from Cargo.toml
get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Create AppImageHub recipe
create_recipe() {
    local version="$1"
    
    log_info "Creating AppImageHub recipe..."
    
    mkdir -p "$APPIMAGEHUB_DIR"
    
    cat > "${APPIMAGEHUB_DIR}/lazyllama.yml" << EOF
app: lazyllama
binpatch: true

ingredients:
  dist: jammy
  sources:
    - deb http://archive.ubuntu.com/ubuntu/ jammy main universe
  packages:
    - libssl3
  script:
    - wget -q "https://github.com/Pommersche92/lazyllama/releases/download/v${version}/lazyllama-${version}-x86_64.AppImage"
    - chmod +x lazyllama-${version}-x86_64.AppImage
    - ./lazyllama-${version}-x86_64.AppImage --appimage-extract
    - mv squashfs-root AppDir

script:
  - cp -r AppDir/usr/share/applications/*.desktop .
  - cp -r AppDir/usr/share/icons/hicolor/256x256/apps/*.png .
EOF
    
    log_success "Recipe created: ${APPIMAGEHUB_DIR}/lazyllama.yml"
    return 0
}

# Create AppData/MetaInfo file
create_appdata() {
    local version="$1"
    
    log_info "Creating AppData metadata..."
    
    cat > "${APPIMAGEHUB_DIR}/lazyllama.appdata.xml" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>app.pommersche.LazyLlama</id>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>GPL-2.0-or-later</project_license>
  <name>LazyLlama</name>
  <summary>A fast LLM client for the terminal</summary>
  
  <description>
    <p>
      LazyLlama is a lightweight, fast Terminal User Interface (TUI) client for Ollama.
      It is designed for running local AI models with minimal overhead and intuitive,
      Emacs-inspired controls directly in your terminal.
    </p>
    <p>Features:</p>
    <ul>
      <li>Real-time streaming responses</li>
      <li>Markdown support with syntax highlighting</li>
      <li>Smart scrolling with autoscroll and manual modes</li>
      <li>Model management with separate buffers per model</li>
      <li>Automatic session logging</li>
      <li>Ultra-low latency and minimal resource footprint</li>
    </ul>
  </description>
  
  <launchable type="desktop-id">lazyllama.desktop</launchable>
  
  <url type="homepage">https://pommersche.github.io/lazyllama/</url>
  <url type="bugtracker">https://github.com/Pommersche92/lazyllama/issues</url>
  <url type="donation">https://github.com/sponsors/Pommersche92</url>
  
  <developer id="app.pommersche">
    <name>Pommersche92</name>
  </developer>
  
  <releases>
    <release version="${version}" date="$(date +%Y-%m-%d)">
      <description>
        <p>Latest release of LazyLlama</p>
      </description>
    </release>
  </releases>
  
  <screenshots>
    <screenshot type="default">
      <image>https://raw.githubusercontent.com/Pommersche92/lazyllama/main/docs/images/screenshot.png</image>
      <caption>LazyLlama in action</caption>
    </screenshot>
  </screenshots>
  
  <categories>
    <category>Utility</category>
    <category>ConsoleOnly</category>
  </categories>
  
  <keywords>
    <keyword>llm</keyword>
    <keyword>ai</keyword>
    <keyword>ollama</keyword>
    <keyword>terminal</keyword>
    <keyword>tui</keyword>
  </keywords>
  
  <content_rating type="oars-1.1" />
</component>
EOF
    
    log_success "AppData created: ${APPIMAGEHUB_DIR}/lazyllama.appdata.xml"
    return 0
}

# Display instructions
show_instructions() {
    local version="$1"
    
    echo ""
    log_step "AppImageHub Submission Instructions"
    echo ""
    
    echo -e "${BOLD}AppImageHub uses a pull request workflow. Follow these steps:${NC}"
    echo ""
    
    echo "1. Fork the AppImageHub repository:"
    echo "   https://github.com/AppImage/appimage.github.io"
    echo ""
    
    echo "2. Clone your fork:"
    echo "   git clone https://github.com/YOUR_USERNAME/appimage.github.io.git"
    echo "   cd appimage.github.io"
    echo ""
    
    echo "3. Create a new branch:"
    echo "   git checkout -b add-lazyllama"
    echo ""
    
    echo "4. Add the recipe file:"
    echo "   mkdir -p database/lazyllama"
    echo "   cp ${APPIMAGEHUB_DIR}/lazyllama.yml database/lazyllama/"
    echo ""
    
    echo "5. (Optional) Add AppData metadata for better presentation:"
    echo "   cp ${APPIMAGEHUB_DIR}/lazyllama.appdata.xml database/lazyllama/"
    echo ""
    
    echo "6. Commit and push:"
    echo "   git add database/lazyllama/"
    echo "   git commit -m \"Add LazyLlama v${version}\""
    echo "   git push origin add-lazyllama"
    echo ""
    
    echo "7. Open a Pull Request:"
    echo "   https://github.com/AppImage/appimage.github.io/pulls"
    echo ""
    
    echo -e "${BOLD}Important Notes:${NC}"
    echo "• The AppImage must be available on GitHub Releases first"
    echo "• AppImageHub will automatically update when you create new releases"
    echo "• Screenshots improve discoverability (add to docs/images/ folder)"
    echo ""
    
    echo -e "${BOLD}Resources:${NC}"
    echo "• AppImageHub: https://appimage.github.io/"
    echo "• Guidelines: https://github.com/AppImage/appimage.github.io#how-to-submit-apps-to-the-catalog"
    echo "• AppData spec: https://www.freedesktop.org/software/appstream/docs/"
    echo ""
}

# Main execution
main() {
    log_step "Preparing AppImageHub Submission"
    echo ""
    
    # Get version
    VERSION=$(get_version)
    log_info "Version: v$VERSION"
    echo ""
    
    # Create files
    create_recipe "$VERSION"
    create_appdata "$VERSION"
    
    # Show instructions
    show_instructions "$VERSION"
    
    log_success "Preparation complete!"
    log_info "Files created in: $APPIMAGEHUB_DIR"
}

# Run main function
main
