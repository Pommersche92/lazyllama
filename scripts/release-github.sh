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

# Parse arguments
DRAFT_MODE=false
RELEASE_NOTES=""
SKIP_BUILD=false

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
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Creates a GitHub release with pre-built binary tarball."
            echo ""
            echo "Options:"
            echo "  --draft              Create as draft release"
            echo "  --notes TEXT         Release notes (optional)"
            echo "  --skip-build         Skip building, use existing binary"
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
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
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

# Create GitHub release
create_github_release() {
    local version="$1"
    local tarball_path="$2"
    local tag="v${version}"
    
    log_info "Creating GitHub release $tag..."
    
    # Build gh release create command
    local gh_cmd="gh release create $tag"
    
    # Add tarball
    gh_cmd="$gh_cmd $tarball_path"
    
    # Add title
    gh_cmd="$gh_cmd --title \"LazyLlama v${version}\""
    
    # Add notes if provided
    if [ -n "$RELEASE_NOTES" ]; then
        gh_cmd="$gh_cmd --notes \"$RELEASE_NOTES\""
    else
        # Generate default notes
        gh_cmd="$gh_cmd --notes \"Release v${version}\""
    fi
    
    # Add draft flag if requested
    if [ "$DRAFT_MODE" = true ]; then
        gh_cmd="$gh_cmd --draft"
    fi
    
    # Execute command
    if eval "$gh_cmd"; then
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
    local tarball_path="$2"
    
    echo ""
    log_success "Release Summary"
    echo ""
    echo "  Version:    v${version}"
    echo "  Tarball:    $(basename $tarball_path)"
    echo "  Size:       $(du -h $tarball_path | cut -f1)"
    echo "  SHA256:     $(sha256sum $tarball_path | awk '{print $1}')"
    echo ""
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
    
    # Build release binary
    if [ "$SKIP_BUILD" = false ]; then
        if ! build_release; then
            exit 1
        fi
        echo ""
    else
        log_warning "Skipping build (using existing binary)"
        echo ""
    fi
    
    # Create tarball
    TARBALL_PATH=$(create_tarball "$VERSION" "$PACKAGE_NAME")
    if [ $? -ne 0 ]; then
        exit 1
    fi
    echo ""
    
    # Create GitHub release
    if ! create_github_release "$VERSION" "$TARBALL_PATH"; then
        exit 1
    fi
    
    # Display summary
    display_summary "$VERSION" "$TARBALL_PATH"
    
    # Next steps
    log_info "Next Steps:"
    echo ""
    echo "  1. Verify the release on GitHub"
    echo "  2. Update AUR packages:"
    echo "     ./scripts/deploy-aur.sh --push"
    echo ""
}

# Run main function
main
