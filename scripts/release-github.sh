#!/usr/bin/env bash
#
# GitHub Release Script for LazyLlama
# Usage: ./scripts/release-github.sh [--draft] [--notes "Release notes"]
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"
RELEASE_DIR="${PROJECT_ROOT}/target/release"
DIST_DIR="${PROJECT_ROOT}/target/dist"
FLATPAK_DIR="${PROJECT_ROOT}/flatpak"

# Parse arguments
DRAFT_MODE=false
RELEASE_NOTES=""
SKIP_BUILD=false
SKIP_WINDOWS=false
SKIP_FLATPAK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --draft)
            DRAFT_MODE=true
            shift
            ;;
        --notes)
            RELEASE_NOTES="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-windows)
            SKIP_WINDOWS=true
            shift
            ;;
        --skip-flatpak)
            SKIP_FLATPAK=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Creates a GitHub release with pre-built binaries (Linux x64, Windows x64, Flatpak)."
            echo ""
            echo "Options:"
            echo "  --draft              Create as draft release"
            echo "  --notes TEXT         Release notes (optional)"
            echo "  --skip-build         Skip building, use existing binary"
            echo "  --skip-windows       Skip Windows cross-compile"
            echo "  --skip-flatpak       Skip Flatpak bundle creation"
            echo "  -h, --help          Show this help message"
            echo ""
            echo "Example:"
            echo "  $0"
            echo "  $0 --draft --notes 'Bug fixes and improvements'"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}✓${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1" >&2
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

# Extract version from Cargo.toml
get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Extract package name from Cargo.toml
get_package_name() {
    grep '^name = ' "$CARGO_TOML" | head -n1 | sed 's/name = "\(.*\)"/\1/'
}

# Check if gh CLI is installed
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) is not installed"
        log_info "Install it with: sudo pacman -S github-cli"
        log_info "Or visit: https://cli.github.com/"
        exit 1
    fi
}

# Check if user is authenticated with gh
check_gh_auth() {
    if ! gh auth status &> /dev/null; then
        log_error "Not authenticated with GitHub CLI"
        log_info "Run: gh auth login"
        exit 1
    fi
}

# Check if release already exists
check_release_exists() {
    local version="$1"
    local tag="v${version}"
    
    if gh release view "$tag" &> /dev/null; then
        log_error "Release $tag already exists"
        log_info "Delete it first with: gh release delete $tag"
        return 1
    fi
    return 0
}

# Build release binary
build_release() {
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

# Build Windows binary (cross-compile)
build_windows() {
    log_info "Building Windows x64 binary..."
    
    cd "$PROJECT_ROOT"
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
        log_info "Installing Windows target..."
        if ! rustup target add x86_64-pc-windows-gnu; then
            log_error "Failed to install Windows target"
            return 1
        fi
    fi
    
    # Check if mingw-w64 is installed
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        log_warning "mingw-w64-gcc not found"
        log_info "Install with: sudo pacman -S mingw-w64-gcc"
        return 1
    fi
    
    # Build for Windows
    if cargo build --release --target x86_64-pc-windows-gnu; then
        log_success "Windows binary built successfully"
        return 0
    else
        log_error "Failed to build Windows binary"
        return 1
    fi
}

# Build Flatpak bundle
build_flatpak_bundle() {
    log_info "Building Flatpak bundle..."
    
    cd "$PROJECT_ROOT"
    
    # Check if flatpak-builder is installed
    if ! command -v flatpak-builder &> /dev/null; then
        log_warning "flatpak-builder not found"
        log_info "Install with: sudo pacman -S flatpak-builder"
        return 1
    fi
    
    # Run flatpak build script
    if ! "${PROJECT_ROOT}/scripts/build-flatpak.sh" build; then
        log_error "Failed to build Flatpak"
        return 1
    fi
    
    # Create bundle
    local version=$(get_version)
    local bundle_name="lazyllama-${version}-x86_64.flatpak"
    local bundle_path="${DIST_DIR}/${bundle_name}"
    
    mkdir -p "$DIST_DIR"
    
    if flatpak build-bundle flatpak-repo "$bundle_path" app.pommersche.LazyLlama; then
        log_success "Flatpak bundle created: $bundle_name"
        echo "$bundle_path"
        return 0
    else
        log_error "Failed to create Flatpak bundle"
        return 1
    fi
}

# Create distribution tarball
create_tarball() {
    local version="$1"
    local pkg_name="$2"
    local tarball_name="${pkg_name}-${version}-x86_64.tar.gz"
    local tarball_path="${DIST_DIR}/${tarball_name}"
    
    log_info "Creating distribution tarball..."
    
    # Create dist directory
    mkdir -p "$DIST_DIR"
    
    # Create temporary directory for tarball contents
    local temp_dir=$(mktemp -d)
    
    # Copy binary
    if [ ! -f "${RELEASE_DIR}/${pkg_name}" ]; then
        log_error "Binary not found: ${RELEASE_DIR}/${pkg_name}"
        rm -rf "$temp_dir"
        return 1
    fi
    cp "${RELEASE_DIR}/${pkg_name}" "$temp_dir/"
    
    # Copy LICENSE if exists
    if [ -f "${PROJECT_ROOT}/LICENSE" ]; then
        cp "${PROJECT_ROOT}/LICENSE" "$temp_dir/"
    fi
    
    # Copy README if exists
    if [ -f "${PROJECT_ROOT}/README.md" ]; then
        cp "${PROJECT_ROOT}/README.md" "$temp_dir/"
    fi
    
    # Create tarball
    cd "$temp_dir"
    if tar -czf "$tarball_path" *; then
        log_success "Tarball created: $tarball_name"
        
        # Calculate and display SHA256
        local checksum=$(sha256sum "$tarball_path" | awk '{print $1}')
        log_info "SHA256: $checksum"
        
        rm -rf "$temp_dir"
        echo "$tarball_path"
        return 0
    else
        log_error "Failed to create tarball"
        rm -rf "$temp_dir"
        return 1
    fi
}

# Create Windows ZIP
create_windows_zip() {
    local version="$1"
    local pkg_name="$2"
    local zip_name="${pkg_name}-${version}-x86_64-windows.zip"
    local zip_path="${DIST_DIR}/${zip_name}"
    
    log_info "Creating Windows distribution ZIP..."
    
    # Create dist directory
    mkdir -p "$DIST_DIR"
    
    # Create temporary directory for ZIP contents
    local temp_dir=$(mktemp -d)
    
    # Copy binary
    local windows_binary="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/${pkg_name}.exe"
    if [ ! -f "$windows_binary" ]; then
        log_error "Windows binary not found: $windows_binary"
        rm -rf "$temp_dir"
        return 1
    fi
    cp "$windows_binary" "$temp_dir/"
    
    # Copy LICENSE if exists
    if [ -f "${PROJECT_ROOT}/LICENSE" ]; then
        cp "${PROJECT_ROOT}/LICENSE" "$temp_dir/"
    fi
    
    # Copy README if exists
    if [ -f "${PROJECT_ROOT}/README.md" ]; then
        cp "${PROJECT_ROOT}/README.md" "$temp_dir/"
    fi
    
    # Create ZIP
    cd "$temp_dir"
    if zip -r "$zip_path" *; then
        log_success "ZIP created: $zip_name"
        
        # Calculate and display SHA256
        local checksum=$(sha256sum "$zip_path" | awk '{print $1}')
        log_info "SHA256: $checksum"
        
        rm -rf "$temp_dir"
        echo "$zip_path"
        return 0
    else
        log_error "Failed to create ZIP"
        rm -rf "$temp_dir"
        return 1
    fi
}

# Create GitHub release
create_github_release() {
    local version="$1"
    shift  # Remove first argument, rest are file paths
    local files=("$@")
    local tag="v${version}"
    
    log_info "Creating GitHub release $tag..."
    
    # Build command arguments array
    local gh_args=(
        "release" "create" "$tag"
    )
    
    # Add all files
    for file in "${files[@]}"; do
        gh_args+=("$file")
    done
    
    # Add title
    gh_args+=("--title" "LazyLlama v${version}")
    
    # Add notes
    if [ -n "$RELEASE_NOTES" ]; then
        gh_args+=("--notes" "$RELEASE_NOTES")
    else
        gh_args+=("--notes" "Release v${version}")
    fi
    
    # Add draft flag if requested
    if [ "$DRAFT_MODE" = true ]; then
        gh_args+=("--draft")
    fi
    
    # Execute command
    if gh "${gh_args[@]}"; then
        log_success "GitHub release created successfully"
        
        if [ "$DRAFT_MODE" = true ]; then
            log_info "Release created as DRAFT"
            log_info "Edit and publish at: https://github.com/Pommersche92/lazyllama/releases"
        else
            log_info "View release at: https://github.com/Pommersche92/lazyllama/releases/tag/$tag"
        fi
        return 0
    else
        log_error "Failed to create GitHub release"
        return 1
    fi
}

# Display summary
display_summary() {
    local version="$1"
    shift  # Remove first argument, rest are file paths
    local files=("$@")
    
    echo ""
    log_success "Release Summary"
    echo ""
    echo "  Version:    v${version}"
    echo ""
    echo "  Assets:"
    for file in "${files[@]}"; do
        echo "    - $(basename $file)"
        echo "      Size:    $(du -h $file | cut -f1)"
        echo "      SHA256:  $(sha256sum $file | awk '{print $1}')"
        echo ""
    done
}

# Main execution
main() {
    log_info "LazyLlama GitHub Release"
    echo ""
    
    # Checks
    check_gh_cli
    check_gh_auth
    
    # Get version and package name
    VERSION=$(get_version)
    PACKAGE_NAME=$(get_package_name)
    
    log_info "Package: $PACKAGE_NAME"
    log_info "Version: $VERSION"
    echo ""
    
    # Check if release exists
    if ! check_release_exists "$VERSION"; then
        exit 1
    fi
    
    # Array to collect all release assets
    local release_assets=()
    
    # Build Linux x64 binary
    if [ "$SKIP_BUILD" = false ]; then
        if ! build_release; then
            exit 1
        fi
        echo ""
    else
        log_warning "Skipping Linux build (using existing binary)"
        echo ""
    fi
    
    # Create Linux tarball
    TARBALL_PATH=$(create_tarball "$VERSION" "$PACKAGE_NAME")
    if [ $? -ne 0 ]; then
        exit 1
    fi
    release_assets+=("$TARBALL_PATH")
    echo ""
    
    # Build Windows x64 binary
    if [ "$SKIP_WINDOWS" = false ]; then
        if build_windows; then
            echo ""
            
            # Create Windows ZIP
            WINDOWS_ZIP=$(create_windows_zip "$VERSION" "$PACKAGE_NAME")
            if [ $? -ne 0 ]; then
                log_warning "Skipping Windows ZIP (build failed)"
            else
                release_assets+=("$WINDOWS_ZIP")
            fi
            echo ""
        else
            log_warning "Skipping Windows build (dependencies not met)"
            echo ""
        fi
    else
        log_warning "Skipping Windows build (--skip-windows)"
        echo ""
    fi
    
    # Build Flatpak bundle
    if [ "$SKIP_FLATPAK" = false ]; then
        FLATPAK_BUNDLE=$(build_flatpak_bundle)
        if [ $? -ne 0 ]; then
            log_warning "Skipping Flatpak bundle (build failed)"
        else
            release_assets+=("$FLATPAK_BUNDLE")
        fi
        echo ""
    else
        log_warning "Skipping Flatpak build (--skip-flatpak)"
        echo ""
    fi
    
    # Create GitHub release
    if ! create_github_release "$VERSION" "${release_assets[@]}"; then
        exit 1
    fi
    
    # Display summary
    display_summary "$VERSION" "${release_assets[@]}"
    
    # Next steps
    log_info "Next Steps:"
    echo ""
    echo "  1. Verify the release on GitHub"
    echo "  2. Update AUR packages:"
    echo "     ./scripts/deploy-aur.sh --push"
    echo "  3. Update Flatpak manifest:"
    echo "     ./scripts/deploy-flathub.sh update"
    echo ""
}

# Run main function
main
