#!/usr/bin/env bash
#
# AUR Deployment Script for LazyLlama
# Usage: ./scripts/deploy-aur.sh [--push] [--package lazyllama|lazyllama-bin]
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
AUR_DIR="${PROJECT_ROOT}/aur"

# Parse arguments
PUSH_TO_AUR=false
SPECIFIC_PACKAGE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH_TO_AUR=true
            shift
            ;;
        --package)
            SPECIFIC_PACKAGE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--push] [--package lazyllama|lazyllama-bin]"
            echo ""
            echo "Options:"
            echo "  --push                Push to AUR after updating PKGBUILDs"
            echo "  --package NAME        Only process specified package (lazyllama or lazyllama-bin)"
            echo "  -h, --help           Show this help message"
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

# Update PKGBUILD version
update_pkgbuild_version() {
    local pkgbuild_path="$1"
    local new_version="$2"
    
    sed -i "s/^pkgver=.*/pkgver=$new_version/" "$pkgbuild_path"
    sed -i "s/^pkgrel=.*/pkgrel=1/" "$pkgbuild_path"  # Reset pkgrel on version bump
}

# Calculate and update SHA256 sum for source package
update_source_checksum() {
    local pkgbuild_path="$1"
    local version="$2"
    local pkgname="lazyllama"
    
    log_info "Downloading crate from crates.io to calculate checksum..."
    
    local crate_url="https://static.crates.io/crates/${pkgname}/${pkgname}-${version}.crate"
    local temp_file=$(mktemp)
    
    if curl -fsSL "$crate_url" -o "$temp_file"; then
        local checksum=$(sha256sum "$temp_file" | awk '{print $1}')
        sed -i "s/^sha256sums=.*/sha256sums=('$checksum')/" "$pkgbuild_path"
        rm "$temp_file"
        log_success "Updated checksum for source package: $checksum"
        return 0
    else
        rm "$temp_file"
        log_error "Failed to download crate from crates.io"
        log_warning "Make sure version $version is published on crates.io first!"
        return 1
    fi
}

# Calculate and update SHA256 sum for binary package
update_binary_checksum() {
    local pkgbuild_path="$1"
    local version="$2"
    
    log_info "Downloading release binary from GitHub to calculate checksum..."
    
    local release_url="https://github.com/Pommersche92/lazyllama/releases/download/v${version}/lazyllama-${version}-x86_64.tar.gz"
    local temp_file=$(mktemp)
    
    if curl -fsSL "$release_url" -o "$temp_file"; then
        local checksum=$(sha256sum "$temp_file" | awk '{print $1}')
        sed -i "s/^sha256sums=.*/sha256sums=('$checksum')/" "$pkgbuild_path"
        rm "$temp_file"
        log_success "Updated checksum for binary package: $checksum"
        return 0
    else
        rm "$temp_file"
        log_error "Failed to download release from GitHub"
        log_warning "Make sure version v$version is published on GitHub releases first!"
        log_warning "Binary should be named: lazyllama-${version}-x86_64.tar.gz"
        return 1
    fi
}

# Test build PKGBUILD
test_build() {
    local pkg_dir="$1"
    local pkg_name=$(basename "$pkg_dir")
    
    log_info "Testing build for $pkg_name..."
    
    cd "$pkg_dir"
    
    # Use a temporary build directory
    local build_dir=$(mktemp -d)
    cp PKGBUILD "$build_dir/"
    cd "$build_dir"
    
    if makepkg --printsrcinfo > .SRCINFO 2>/dev/null; then
        log_success "PKGBUILD is valid and .SRCINFO generated"
        rm -rf "$build_dir"
        return 0
    else
        log_error "PKGBUILD validation failed"
        rm -rf "$build_dir"
        return 1
    fi
}

# Push to AUR
push_to_aur() {
    local pkg_dir="$1"
    local pkg_name=$(basename "$pkg_dir")
    local version="$2"
    local aur_repo_dir="${pkg_dir}/aur-repo"
    
    log_info "Pushing $pkg_name to AUR..."
    
    cd "$pkg_dir"
    
    # Generate .SRCINFO
    makepkg --printsrcinfo > .SRCINFO
    
    # Check if AUR repository exists
    if [ -d "$aur_repo_dir/.git" ]; then
        log_info "Copying files to AUR repository..."
        cp PKGBUILD .SRCINFO "$aur_repo_dir/"
        
        cd "$aur_repo_dir"
        
        # Check if there are changes
        if git diff --quiet PKGBUILD .SRCINFO 2>/dev/null; then
            log_warning "No changes detected, skipping push"
            return 0
        fi
        
        log_info "Committing and pushing to AUR..."
        git add PKGBUILD .SRCINFO
        git commit -m "Update to version $version"
        git push
        log_success "Pushed $pkg_name to AUR"
    else
        log_warning "No git repository found in $aur_repo_dir"
        log_info "To set up AUR repository:"
        echo "" >&2
        echo "  cd $pkg_dir" >&2
        echo "  git clone ssh://aur@aur.archlinux.org/${pkg_name}.git aur-repo" >&2
        echo "  cp PKGBUILD .SRCINFO aur-repo/" >&2
        echo "  cd aur-repo" >&2
        echo "  git add PKGBUILD .SRCINFO" >&2
        echo "  git commit -m 'Initial commit: version $version'" >&2
        echo "  git push" >&2
        echo "" >&2
    fi
}

# Process a single package
process_package() {
    local pkg_name="$1"
    local pkg_dir="${AUR_DIR}/${pkg_name}"
    local pkgbuild="${pkg_dir}/PKGBUILD"
    
    log_info "Processing $pkg_name..."
    
    if [ ! -f "$pkgbuild" ]; then
        log_error "PKGBUILD not found: $pkgbuild"
        return 1
    fi
    
    # Update version
    update_pkgbuild_version "$pkgbuild" "$VERSION"
    log_success "Updated version to $VERSION in $pkg_name/PKGBUILD"
    
    # Update checksum based on package type
    if [[ "$pkg_name" == "lazyllama-bin" ]]; then
        if ! update_binary_checksum "$pkgbuild" "$VERSION"; then
            return 1
        fi
    else
        if ! update_source_checksum "$pkgbuild" "$VERSION"; then
            return 1
        fi
    fi
    
    # Test build
    if ! test_build "$pkg_dir"; then
        log_error "Build test failed for $pkg_name"
        return 1
    fi
    
    # Push to AUR if requested
    if [ "$PUSH_TO_AUR" = true ]; then
        push_to_aur "$pkg_dir" "$VERSION"
    fi
    
    log_success "Successfully processed $pkg_name"
    echo ""
    return 0
}

# Main execution
main() {
    log_info "LazyLlama AUR Deployment"
    echo ""
    
    # Get version
    VERSION=$(get_version)
    log_info "Current version: $VERSION"
    echo ""
    
    # Determine which packages to process
    if [ -n "$SPECIFIC_PACKAGE" ]; then
        PACKAGES=("$SPECIFIC_PACKAGE")
    else
        PACKAGES=("lazyllama" "lazyllama-bin")
    fi
    
    # Process packages
    local failed=0
    for pkg in "${PACKAGES[@]}"; do
        if ! process_package "$pkg"; then
            failed=$((failed + 1))
        fi
    done
    
    # Summary
    echo ""
    if [ $failed -eq 0 ]; then
        log_success "All packages processed successfully!"
        
        if [ "$PUSH_TO_AUR" = false ]; then
            echo ""
            log_info "PKGBUILDs updated. To push to AUR, run:"
            echo "  $0 --push"
        fi
    else
        log_error "$failed package(s) failed to process"
        exit 1
    fi
}

# Run main function
main
