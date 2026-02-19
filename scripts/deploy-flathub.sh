#!/usr/bin/env bash
#
# Flathub Deployment Script for LazyLlama
# Prepares and submits to Flathub
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
FLATPAK_DIR="${PROJECT_ROOT}/flatpak"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"
FLATHUB_REPO_DIR="${PROJECT_ROOT}/flathub-repo"

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

# Get version
get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Initial Flathub submission
submit_initial() {
    log_step "Initial Flathub Submission"
    echo "" >&2
    
    log_info "This will guide you through submitting LazyLlama to Flathub"
    echo "" >&2
    
    # Check if Flathub repo exists
    if [ -d "$FLATHUB_REPO_DIR" ]; then
        log_warning "Flathub repository already exists at $FLATHUB_REPO_DIR"
        echo -n "Delete and start fresh? (y/N): " >&2
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            rm -rf "$FLATHUB_REPO_DIR"
        else
            log_info "Using existing repository"
        fi
    fi
    
    if [ ! -d "$FLATHUB_REPO_DIR" ]; then
        log_info "Creating new Flathub repository..."
        mkdir -p "$FLATHUB_REPO_DIR"
        cd "$FLATHUB_REPO_DIR"
        git init
        git checkout -b branch/lazyllama
    fi
    
    # Copy files
    log_info "Copying Flatpak files..."
    cp "${FLATPAK_DIR}/app.pommersche.LazyLlama.json" "$FLATHUB_REPO_DIR/"
    cp "${FLATPAK_DIR}/app.pommersche.LazyLlama.metainfo.xml" "$FLATHUB_REPO_DIR/"
    cp "${FLATPAK_DIR}/app.pommersche.LazyLlama.desktop" "$FLATHUB_REPO_DIR/"
    
    # Generate sources if not exists
    if [ ! -f "${FLATPAK_DIR}/generated-sources.json" ]; then
        log_warning "generated-sources.json not found"
        log_info "Run: ./scripts/build-flatpak.sh setup"
        return 1
    fi
    
    cp "${FLATPAK_DIR}/generated-sources.json" "$FLATHUB_REPO_DIR/"
    
    log_success "Files copied"
    echo "" >&2
    
    # Instructions
    log_step "Next Steps for Initial Submission"
    echo "" >&2
    echo "1. Create a GitHub account and fork: https://github.com/flathub/flathub" >&2
    echo "" >&2
    echo "2. Create a new repository on GitHub:" >&2
    echo "   Name: app.pommersche.LazyLlama" >&2
    echo "   Description: LazyLlama Flatpak for Flathub" >&2
    echo "" >&2
    echo "3. Push your Flatpak files:" >&2
    echo "   cd $FLATHUB_REPO_DIR" >&2
    echo "   git add ." >&2
    echo "   git commit -m 'Initial LazyLlama Flatpak'" >&2
    echo "   git remote add origin git@github.com:YOUR_USERNAME/app.pommersche.LazyLlama.git" >&2
    echo "   git push -u origin branch/lazyllama" >&2
    echo "" >&2
    echo "4. Create Pull Request to Flathub:" >&2
    echo "   https://github.com/flathub/flathub/pulls" >&2
    echo "   Submit PR to merge your repository" >&2
    echo "" >&2
    echo "5. Flathub review team will review and approve" >&2
    echo "" >&2
    
    log_info "Repository prepared at: $FLATHUB_REPO_DIR"
}

# Update existing Flathub package
update_flathub() {
    local version="$1"
    
    log_step "Updating Flathub Package"
    echo "" >&2
    
    # Check if Flathub repo exists
    if [ ! -d "${FLATHUB_REPO_DIR}/.git" ]; then
        log_error "Flathub repository not found"
        log_info "For initial submission, run: $0 submit"
        log_info "For existing package, clone: git clone https://github.com/flathub/app.pommersche.LazyLlama.git $FLATHUB_REPO_DIR"
        return 1
    fi
    
    cd "$FLATHUB_REPO_DIR"
    
    # Update files
    log_info "Updating Flatpak files..."
    
    "${PROJECT_ROOT}/scripts/build-flatpak.sh" update
    
    cp "${FLATPAK_DIR}/app.pommersche.LazyLlama.json" .
    cp "${FLATPAK_DIR}/app.pommersche.LazyLlama.metainfo.xml" .
    cp "${FLATPAK_DIR}/generated-sources.json" .
    
    # Update metainfo release
    local today=$(date +%Y-%m-%d)
    sed -i "/<releases>/a\\    <release version=\"$version\" date=\"$today\">\n      <description>\n        <p>Release v$version</p>\n      </description>\n    </release>" \
        app.pommersche.LazyLlama.metainfo.xml
    
    log_success "Files updated"
    echo "" >&2
    
    # Git operations
    log_info "Creating git commit..."
    
    git checkout -b "update-v${version}" || git checkout "update-v${version}"
    git add .
    git commit -m "Update to version $version"
    
    log_success "Committed changes"
    echo "" >&2
    
    log_info "Next steps:"
    echo "" >&2
    echo "1. Push to your fork:" >&2
    echo "   cd $FLATHUB_REPO_DIR" >&2
    echo "   git push -u origin update-v${version}" >&2
    echo "" >&2
    echo "2. Create Pull Request:" >&2
    echo "   https://github.com/flathub/app.pommersche.LazyLlama/pulls" >&2
    echo "" >&2
}

# Main
main() {
    local command="${1:-}"
    
    case "$command" in
        submit|initial)
            submit_initial
            ;;
        update)
            VERSION=$(get_version)
            update_flathub "$VERSION"
            ;;
        -h|--help)
            echo "Usage: $0 [submit|update]"
            echo ""
            echo "Commands:"
            echo "  submit    Initial Flathub submission (first time)"
            echo "  update    Update existing Flathub package"
            echo ""
            echo "Examples:"
            echo "  $0 submit    # First-time submission"
            echo "  $0 update    # Update to new version"
            exit 0
            ;;
        *)
            log_error "Unknown command: $command"
            echo "Run with --help for usage" >&2
            exit 1
            ;;
    esac
}

main "$@"
