#!/usr/bin/env bash
#
# Flatpak Build Script for LazyLlama
# Generates cargo dependencies and builds/tests the Flatpak
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FLATPAK_DIR="${PROJECT_ROOT}/flatpak"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"

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

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing=()
    
    if ! command -v flatpak &> /dev/null; then
        missing+=("flatpak")
    fi
    
    if ! command -v flatpak-builder &> /dev/null; then
        missing+=("flatpak-builder")
    fi
    
    if ! command -v python3 &> /dev/null; then
        missing+=("python3")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        log_info "Install with: sudo pacman -S flatpak flatpak-builder python"
        return 1
    fi
    
    log_success "All dependencies installed"
    return 0
}

# Setup Flatpak
setup_flatpak() {
    log_info "Setting up Flatpak repositories..."
    
    if ! flatpak remote-list | grep -q flathub; then
        flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
    fi
    
    log_info "Installing Flatpak SDK..."
    flatpak install -y flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08 org.freedesktop.Sdk.Extension.rust-stable//23.08 || true
    
    log_success "Flatpak setup complete"
}

# Download flatpak-cargo-generator
download_cargo_generator() {
    local generator_path="${FLATPAK_DIR}/flatpak-cargo-generator.py"
    
    if [ -f "$generator_path" ]; then
        log_info "Cargo generator already exists"
        return 0
    fi
    
    log_info "Downloading flatpak-cargo-generator..."
    
    curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py \
        -o "$generator_path"
    
    chmod +x "$generator_path"
    
    log_success "Downloaded cargo generator"
}

# Install Python dependencies for cargo generator
install_python_dependencies() {
    log_info "Checking Python dependencies..."
    
    # Check if dependencies are already installed
    if python3 -c "import tomlkit, aiohttp, yaml" 2>/dev/null; then
        log_success "Python dependencies already installed"
        return 0
    fi
    
    log_warning "Missing Python dependencies for flatpak-cargo-generator"
    log_info "Please install them with:"
    echo ""
    echo "  sudo pacman -S python-tomlkit python-aiohttp python-yaml"
    echo ""
    log_info "Or create a virtual environment:"
    echo ""
    echo "  python -m venv flatpak/.venv"
    echo "  source flatpak/.venv/bin/activate"
    echo "  pip install tomlkit aiohttp PyYAML"
    echo ""
    
    return 1
}

# Generate cargo sources
generate_cargo_sources() {
    log_info "Generating Cargo dependencies..."
    
    # Ensure Python dependencies are installed
    install_python_dependencies || return 1
    
    cd "$FLATPAK_DIR"
    
    python3 flatpak-cargo-generator.py ../Cargo.lock -o generated-sources.json
    
    log_success "Generated Cargo sources"
}

# Update manifest with current version
update_manifest() {
    local version="$1"
    log_info "Updating manifest to version $version..."
    
    # Download source tarball to calculate SHA256
    local temp_file=$(mktemp)
    local url="https://github.com/Pommersche92/lazyllama/archive/refs/tags/v${version}.tar.gz"
    
    if curl -fsSL "$url" -o "$temp_file"; then
        local checksum=$(sha256sum "$temp_file" | awk '{print $1}')
        rm "$temp_file"
        
        # Update manifest
        sed -i "s|\"url\": \"https://github.com/Pommersche92/lazyllama/archive/refs/tags/v.*\.tar\.gz\"|\"url\": \"$url\"|" \
            "${FLATPAK_DIR}/app.pommersche.LazyLlama.json"
        sed -i "s|\"sha256\": \".*\"|\"sha256\": \"$checksum\"|" \
            "${FLATPAK_DIR}/app.pommersche.LazyLlama.json"
        
        log_success "Updated manifest with version $version (SHA256: $checksum)"
    else
        log_error "Failed to download source tarball"
        log_warning "Make sure GitHub release v$version exists"
        return 1
    fi
}

# Build Flatpak
build_flatpak() {
    log_info "Building Flatpak..."
    
    cd "$PROJECT_ROOT"
    
    # Clean previous build
    rm -rf flatpak-build flatpak-repo
    
    # Build
    flatpak-builder --force-clean --disable-rofiles-fuse \
        flatpak-build flatpak/app.pommersche.LazyLlama.json
    
    log_success "Flatpak built successfully"
}

# Test Flatpak
test_flatpak() {
    log_info "Testing Flatpak..."
    
    cd "$PROJECT_ROOT"
    
    flatpak-builder --run flatpak-build flatpak/app.pommersche.LazyLlama.json lazyllama --version || true
    
    log_success "Flatpak test complete"
}

# Install Flatpak locally
install_flatpak() {
    log_info "Installing Flatpak locally..."
    
    cd "$PROJECT_ROOT"
    
    # Export to local repo
    if [ ! -d flatpak-repo ]; then
        flatpak-builder --repo=flatpak-repo --force-clean flatpak-build flatpak/app.pommersche.LazyLlama.json
    fi
    
    # Install
    flatpak --user remote-add --if-not-exists --no-gpg-verify lazyllama-repo flatpak-repo
    flatpak --user install -y lazyllama-repo app.pommersche.LazyLlama
    
    log_success "Installed locally"
    log_info "Run with: flatpak run app.pommersche.LazyLlama"
}

# Main menu
show_menu() {
    echo ""
    echo -e "${BLUE}LazyLlama Flatpak Builder${NC}"
    echo ""
    echo "1. Setup (install dependencies + generate sources)"
    echo "2. Build (build Flatpak)"
    echo "3. Test (run Flatpak)"
    echo "4. Install (install locally)"
    echo "5. Full pipeline (setup + build + test + install)"
    echo "6. Update version (update manifest from Cargo.toml)"
    echo "0. Exit"
    echo ""
    echo -n "Choose option: "
}

# Parse arguments
if [ $# -gt 0 ]; then
    case "$1" in
        setup)
            check_dependencies
            setup_flatpak
            download_cargo_generator
            generate_cargo_sources
            ;;
        build)
            build_flatpak
            ;;
        test)
            test_flatpak
            ;;
        install)
            install_flatpak
            ;;
        update)
            VERSION=$(get_version)
            update_manifest "$VERSION"
            ;;
        full|all)
            VERSION=$(get_version)
            check_dependencies
            setup_flatpak
            download_cargo_generator
            generate_cargo_sources
            update_manifest "$VERSION"
            build_flatpak
            test_flatpak
            install_flatpak
            ;;
        -h|--help)
            echo "Usage: $0 [setup|build|test|install|update|full]"
            echo ""
            echo "Commands:"
            echo "  setup     Install dependencies and generate sources"
            echo "  build     Build Flatpak"
            echo "  test      Test Flatpak"
            echo "  install   Install Flatpak locally"
            echo "  update    Update manifest with current version"
            echo "  full      Complete pipeline"
            exit 0
            ;;
        *)
            log_error "Unknown command: $1"
            echo "Run with --help for usage"
            exit 1
            ;;
    esac
else
    # Interactive menu
    while true; do
        show_menu
        read -r choice
        
        case $choice in
            1)
                check_dependencies
                setup_flatpak
                download_cargo_generator
                generate_cargo_sources
                ;;
            2)
                build_flatpak
                ;;
            3)
                test_flatpak
                ;;
            4)
                install_flatpak
                ;;
            5)
                VERSION=$(get_version)
                check_dependencies
                setup_flatpak
                download_cargo_generator
                generate_cargo_sources
                update_manifest "$VERSION"
                build_flatpak
                test_flatpak
                install_flatpak
                ;;
            6)
                VERSION=$(get_version)
                update_manifest "$VERSION"
                ;;
            0)
                echo "Goodbye!"
                exit 0
                ;;
            *)
                log_error "Invalid option"
                ;;
        esac
        
        echo ""
        echo "Press Enter to continue..."
        read -r
    done
fi
