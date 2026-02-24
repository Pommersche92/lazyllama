#!/usr/bin/env bash
#
# AppImage Build Script for LazyLlama
# Creates a portable AppImage package
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"
BUILD_DIR="${PROJECT_ROOT}/target/appimage-build"
APPDIR="${BUILD_DIR}/AppDir"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"

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

# Get version from Cargo.toml
get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Get package name from Cargo.toml
get_package_name() {
    grep '^name = ' "$CARGO_TOML" | head -n1 | sed 's/name = "\(.*\)"/\1/'
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing=()
    
    if ! command -v cargo &> /dev/null; then
        missing+=("cargo")
    fi
    
    if ! command -v wget &> /dev/null; then
        missing+=("wget")
    fi
    
    if ! command -v patchelf &> /dev/null; then
        missing+=("patchelf")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        log_info "Install with: sudo pacman -S cargo wget patchelf"
        return 1
    fi
    
    log_success "All dependencies installed"
    return 0
}

# Download linuxdeploy
download_linuxdeploy() {
    local linuxdeploy_path="${BUILD_DIR}/linuxdeploy-x86_64.AppImage"
    
    if [ -f "$linuxdeploy_path" ]; then
        log_info "linuxdeploy already exists"
        return 0
    fi
    
    log_info "Downloading linuxdeploy..."
    
    mkdir -p "$BUILD_DIR"
    
    if wget -q "$LINUXDEPLOY_URL" -O "$linuxdeploy_path"; then
        chmod +x "$linuxdeploy_path"
        log_success "Downloaded linuxdeploy"
        return 0
    else
        log_error "Failed to download linuxdeploy"
        return 1
    fi
}

# Build release binary
build_binary() {
    log_info "Building release binary..."
    
    cd "$PROJECT_ROOT"
    
    if cargo build --release; then
        log_success "Release binary built successfully"
        return 0
    else
        log_error "Failed to build release binary"
        return 1
    fi
}

# Create AppDir structure
create_appdir() {
    local package_name="$1"
    local version="$2"
    
    log_info "Creating AppDir structure..."
    
    # Clean and create AppDir
    rm -rf "$APPDIR"
    mkdir -p "${APPDIR}/usr/bin"
    mkdir -p "${APPDIR}/usr/share/applications"
    mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
    
    # Copy binary
    cp "${PROJECT_ROOT}/target/release/${package_name}" "${APPDIR}/usr/bin/"
    
    # Create desktop file
    cat > "${APPDIR}/usr/share/applications/${package_name}.desktop" << EOF
[Desktop Entry]
Type=Application
Name=LazyLlama
Comment=A fast LLM client for the terminal
Exec=${package_name}
Icon=${package_name}
Categories=Utility;ConsoleOnly;
Terminal=true
EOF
    
    # Create a simple icon (PNG) - placeholder
    # For a real icon, you would copy an actual icon file
    # For now, we'll skip the icon or use a simple placeholder
    if [ -f "${PROJECT_ROOT}/docs/images/logo.png" ]; then
        cp "${PROJECT_ROOT}/docs/images/logo.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${package_name}.png"
    else
        log_warning "No icon found, AppImage will have default icon"
    fi
    
    # Create AppRun
    cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/lazyllama" "$@"
EOF
    
    chmod +x "${APPDIR}/AppRun"
    
    # Create .DirIcon symlink
    if [ -f "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${package_name}.png" ]; then
        ln -sf "usr/share/icons/hicolor/256x256/apps/${package_name}.png" "${APPDIR}/.DirIcon"
    fi
    
    # Create desktop file symlink
    ln -sf "usr/share/applications/${package_name}.desktop" "${APPDIR}/${package_name}.desktop"
    
    log_success "AppDir structure created"
    return 0
}

# Build AppImage
build_appimage() {
    local package_name="$1"
    local version="$2"
    local output_name="${package_name}-${version}-x86_64.AppImage"
    local output_path="${PROJECT_ROOT}/target/dist/${output_name}"
    
    log_info "Building AppImage..."
    
    mkdir -p "${PROJECT_ROOT}/target/dist"
    
    cd "$BUILD_DIR"
    
    # Set environment variables for linuxdeploy
    export ARCH=x86_64
    export VERSION="$version"
    export OUTPUT="$output_path"
    
    # Run linuxdeploy
    if ./linuxdeploy-x86_64.AppImage --appdir "$APPDIR" --output appimage; then
        # linuxdeploy creates the AppImage in the current directory
        # Find the created AppImage and move it to the correct location
        local created_appimage=$(find . -maxdepth 1 -name "*.AppImage" -type f | head -n1)
        
        if [ -n "$created_appimage" ]; then
            mv "$created_appimage" "$output_path"
            chmod +x "$output_path"
            log_success "AppImage created: $output_name"
            
            # Calculate SHA256
            local checksum=$(sha256sum "$output_path" | awk '{print $1}')
            log_info "SHA256: $checksum"
            
            echo "$output_path"
            return 0
        else
            log_error "AppImage file not found after build"
            return 1
        fi
    else
        log_error "Failed to build AppImage"
        return 1
    fi
}

# Display usage
show_usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build        Build AppImage (default)"
    echo "  clean        Clean build directory"
    echo "  -h, --help   Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 build"
}

# Clean build directory
clean() {
    log_info "Cleaning build directory..."
    rm -rf "$BUILD_DIR"
    log_success "Build directory cleaned"
}

# Main build process
build() {
    log_info "LazyLlama AppImage Build"
    echo ""
    
    # Get package info
    local package_name=$(get_package_name)
    local version=$(get_version)
    
    log_info "Package: $package_name"
    log_info "Version: $version"
    echo ""
    
    # Check dependencies
    if ! check_dependencies; then
        exit 1
    fi
    echo ""
    
    # Download linuxdeploy
    if ! download_linuxdeploy; then
        exit 1
    fi
    echo ""
    
    # Build binary
    if ! build_binary; then
        exit 1
    fi
    echo ""
    
    # Create AppDir
    if ! create_appdir "$package_name" "$version"; then
        exit 1
    fi
    echo ""
    
    # Build AppImage
    if APPIMAGE_PATH=$(build_appimage "$package_name" "$version"); then
        echo ""
        log_success "AppImage build complete!"
        log_info "Output: $APPIMAGE_PATH"
        return 0
    else
        exit 1
    fi
}

# Main execution
main() {
    local command="${1:-build}"
    
    case "$command" in
        build)
            build
            ;;
        clean)
            clean
            ;;
        -h|--help)
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
