#!/usr/bin/env bash
#
# Complete Release Pipeline for LazyLlama
# Usage: ./scripts/release.sh [--draft]
#
# This script automates the complete release process:
# 1. Publishes to crates.io
# 2. Creates GitHub release with binary
# 3. Deploys to AUR (lazyllama and lazyllama-bin)
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"

# Parse arguments
DRAFT_MODE=false
SKIP_CRATES=false
SKIP_GITHUB=false
SKIP_AUR=false
SKIP_FLATPAK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --draft)
            DRAFT_MODE=true
            shift
            ;;
        --skip-crates)
            SKIP_CRATES=true
            shift
            ;;
        --skip-github)
            SKIP_GITHUB=true
            shift
            ;;
        --skip-aur)
            SKIP_AUR=true
            shift
            ;;
        --skip-flatpak)
            SKIP_FLATPAK=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Complete release pipeline: crates.io → GitHub → AUR → Flatpak"
            echo ""
            echo "Options:"
            echo "  --draft              Create GitHub release as draft"
            echo "  --skip-crates        Skip crates.io publish"
            echo "  --skip-github        Skip GitHub release"
            echo "  --skip-aur           Skip AUR deployment"
            echo "  --skip-flatpak       Skip Flatpak manifest update"
            echo "  -h, --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                   # Full release pipeline"
            echo "  $0 --draft           # Create draft GitHub release"
            echo "  $0 --skip-crates     # Skip crates.io (already published)"
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

log_step() {
    echo -e "${CYAN}${BOLD}▶ $1${NC}" >&2
}

separator() {
    echo "" >&2
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" >&2
    echo "" >&2
}

# Extract version from Cargo.toml
get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Confirm action
confirm() {
    local prompt="$1"
    echo -e -n "${YELLOW}❓${NC} $prompt (y/N): " >&2
    read -r response
    case "$response" in
        [yY][eE][sS]|[yY]) 
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Run tests
run_tests() {
    log_step "Running tests..."
    echo ""
    
    if cargo test --all-features; then
        log_success "All tests passed"
        return 0
    else
        log_error "Tests failed"
        return 1
    fi
}

# Publish to crates.io
publish_crates() {
    log_step "Publishing to crates.io..."
    echo ""
    
    if ! confirm "Publish version $VERSION to crates.io?"; then
        log_warning "Skipping crates.io publish"
        return 0
    fi
    
    if cargo publish; then
        log_success "Published to crates.io"
        
        # Wait for crates.io to sync
        log_info "Waiting 30 seconds for crates.io to sync..."
        sleep 30
        
        return 0
    else
        log_error "Failed to publish to crates.io"
        return 1
    fi
}

# Create GitHub release
create_github_release() {
    log_step "Creating GitHub release..."
    echo ""
    
    local extra_args=""
    if [ "$DRAFT_MODE" = true ]; then
        extra_args="--draft"
    fi
    
    if "${PROJECT_ROOT}/scripts/release-github.sh" $extra_args; then
        log_success "GitHub release created"
        
        # Wait a bit for GitHub to process the release
        log_info "Waiting 10 seconds for GitHub to process release..."
        sleep 10
        
        return 0
    else
        log_error "Failed to create GitHub release"
        return 1
    fi
}

# Deploy to AUR
deploy_aur() {
    log_step "Deploying to AUR..."
    echo ""
    
    if ! confirm "Deploy to AUR (lazyllama and lazyllama-bin)?"; then
        log_warning "Skipping AUR deployment"
        return 0
    fi
    
    if "${PROJECT_ROOT}/scripts/deploy-aur.sh" --push; then
        log_success "Deployed to AUR"
        return 0
    else
        log_error "Failed to deploy to AUR"
        return 1
    fi
}

# Update Flatpak manifest
update_flatpak() {
    log_step "Updating Flatpak manifest..."
    echo ""
    
    if ! confirm "Update Flatpak manifest and metainfo?"; then
        log_warning "Skipping Flatpak update"
        return 0
    fi
    
    if "${PROJECT_ROOT}/scripts/deploy-flathub.sh" update; then
        log_success "Flatpak manifest updated"
        log_info "Remember to push changes to flathub repository"
        return 0
    else
        log_error "Failed to update Flatpak"
        return 1
    fi
}

# Display final summary
display_summary() {
    separator
    echo -e "${GREEN}${BOLD}🎉 Release Complete!${NC}"
    separator
    
    echo -e "${BOLD}Version:${NC} v$VERSION"
    echo ""
    
    if [ "$SKIP_CRATES" = false ]; then
        echo -e "${BOLD}📦 crates.io:${NC}"
        echo "   https://crates.io/crates/lazyllama"
        echo ""
    fi
    
    if [ "$SKIP_GITHUB" = false ]; then
        echo -e "${BOLD}🐙 GitHub:${NC}"
        if [ "$DRAFT_MODE" = true ]; then
            echo "   https://github.com/Pommersche92/lazyllama/releases (DRAFT)"
        else
            echo "   https://github.com/Pommersche92/lazyllama/releases/tag/v$VERSION"
        fi
        echo "   Assets: Linux x64, Windows x64, Flatpak bundle"
        echo ""
    fi
    
    if [ "$SKIP_AUR" = false ]; then
        echo -e "${BOLD}🐧 AUR:${NC}"
        echo "   https://aur.archlinux.org/packages/lazyllama"
        echo "   https://aur.archlinux.org/packages/lazyllama-bin"
        echo ""
    fi
    
    if [ "$SKIP_FLATPAK" = false ]; then
        echo -e "${BOLD}📦 Flatpak:${NC}"
        echo "   Manifest updated in flathub-repo/"
        echo "   Remember to push to Flathub repository"
        echo ""
    fi
    
    echo -e "${BOLD}Installation:${NC}"
    echo "   cargo install lazyllama"
    echo "   yay -S lazyllama"
    echo "   yay -S lazyllama-bin"
    echo "   flatpak install flathub app.pommersche.LazyLlama"
    echo ""
}

# Main execution
main() {
    cd "$PROJECT_ROOT"
    
    # Header
    echo ""
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}${BOLD}        🦙 LazyLlama Release Pipeline 🦙${NC}"
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    # Get version
    VERSION=$(get_version)
    log_info "Version: $VERSION"
    echo ""
    
    # Show what will be done
    log_info "Pipeline steps:"
    echo ""
    [ "$SKIP_CRATES" = false ] && echo "  1. ✓ Publish to crates.io"
    [ "$SKIP_CRATES" = true ] && echo "  1. ✗ Skip crates.io"
    [ "$SKIP_GITHUB" = false ] && echo "  2. ✓ Create GitHub release (Linux x64, Windows x64, Flatpak)"
    [ "$SKIP_GITHUB" = true ] && echo "  2. ✗ Skip GitHub release"
    [ "$SKIP_AUR" = false ] && echo "  3. ✓ Deploy to AUR"
    [ "$SKIP_AUR" = true ] && echo "  3. ✗ Skip AUR"
    [ "$SKIP_FLATPAK" = false ] && echo "  4. ✓ Update Flatpak manifest"
    [ "$SKIP_FLATPAK" = true ] && echo "  4. ✗ Skip Flatpak"
    echo ""
    
    separator
    
    # Run tests
    if ! run_tests; then
        log_error "Tests must pass before release"
        exit 1
    fi
    
    separator
    
    # Step 1: Publish to crates.io
    if [ "$SKIP_CRATES" = false ]; then
        if ! publish_crates; then
            exit 1
        fi
        separator
    fi
    
    # Step 2: Create GitHub release
    if [ "$SKIP_GITHUB" = false ]; then
        if ! create_github_release; then
            exit 1
        fi
        separator
    fi
    
    # Step 3: Deploy to AUR
    if [ "$SKIP_AUR" = false ]; then
        if ! deploy_aur; then
            exit 1
        fi
        separator
    fi
    
    # Step 4: Update Flatpak
    if [ "$SKIP_FLATPAK" = false ]; then
        if ! update_flatpak; then
            exit 1
        fi
        separator
    fi
    
    # Summary
    display_summary
}

# Run main function
main
