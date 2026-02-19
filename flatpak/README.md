# Flatpak Distribution

LazyLlama is available as a Flatpak for universal Linux distribution.

## 📦 User Installation

Once published on Flathub, users can install LazyLlama with:

```bash
flatpak install flathub app.pommersche.LazyLlama
flatpak run app.pommersche.LazyLlama
```

## 🔧 Developer Setup

### Prerequisites

```bash
# Arch Linux
sudo pacman -S flatpak flatpak-builder python python-tomlkit python-aiohttp python-yaml

# Ubuntu/Debian
sudo apt install flatpak flatpak-builder python3 python3-pip
pip3 install tomlkit aiohttp PyYAML

# Fedora
sudo dnf install flatpak flatpak-builder python3 python3-pip
pip3 install tomlkit aiohttp PyYAML
```

### Quick Start

```bash
# Setup and build everything
./scripts/build-flatpak.sh full

# Or step-by-step:
./scripts/build-flatpak.sh setup    # Install dependencies
./scripts/build-flatpak.sh build    # Build Flatpak
./scripts/build-flatpak.sh test     # Test Flatpak
./scripts/build-flatpak.sh install  # Install locally
```

### Interactive Mode

```bash
./scripts/build-flatpak.sh
```

Shows interactive menu with options.

## 📂 File Structure

```
flatpak/
├── app.pommersche.LazyLlama.json          # Flatpak manifest
├── app.pommersche.LazyLlama.metainfo.xml  # AppData metadata
├── app.pommersche.LazyLlama.desktop       # Desktop entry
├── generated-sources.json                 # Cargo dependencies (generated)
└── flatpak-cargo-generator.py            # Dependency generator (downloaded)

scripts/
├── build-flatpak.sh      # Build and test locally
└── deploy-flathub.sh     # Deploy to Flathub
```

## 🚀 Deployment Workflow

### Initial Flathub Submission

```bash
# 1. Prepare files
./scripts/build-flatpak.sh setup

# 2. Update version
./scripts/build-flatpak.sh update

# 3. Submit to Flathub
./scripts/deploy-flathub.sh submit
```

This will:
- Generate Cargo dependency manifest
- Create Flathub repository structure
- Provide instructions for GitHub PR

### Updating Existing Package

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml

# 2. Regenerate dependencies
./scripts/build-flatpak.sh setup

# 3. Update Flathub
./scripts/deploy-flathub.sh update
```

## 🏗️ Build Process Explained

### 1. Dependency Generation

Flatpak requires offline builds. The `flatpak-cargo-generator.py` script converts `Cargo.lock` into a Flatpak-compatible source list:

```bash
python3 flatpak/flatpak-cargo-generator.py Cargo.lock -o flatpak/generated-sources.json
```

This creates `generated-sources.json` with all Rust dependencies from crates.io.

### 2. Manifest Structure

**app.pommersche.LazyLlama.json:**
- `runtime`: org.freedesktop.Platform 23.08
- `sdk`: org.freedesktop.Sdk with Rust extension
- `finish-args`: Permissions (network, terminal, etc.)
- `modules`: Build instructions

**Key Permissions:**
- `--share=network`: Required for Ollama connection
- `--share=ipc`: Inter-process communication
- `--socket=wayland/x11`: For clipboard support
- `--filesystem=host`: File access (may restrict in production)

### 3. Build Steps

```json
"build-commands": [
  "cargo --offline fetch --manifest-path Cargo.toml --verbose",
  "cargo --offline build --release --verbose",
  "install -Dm755 ./target/release/lazyllama -t /app/bin/"
]
```

Builds completely offline using pre-downloaded dependencies.

## 📋 Flathub Submission Guide

### First-Time Submission

1. **Fork Flathub:**
   - Go to https://github.com/flathub/flathub
   - Click "Fork"

2. **Create App Repository:**
   - Create new repo: `app.pommersche.LazyLlama`
   - Push Flatpak files (created by `deploy-flathub.sh submit`)

3. **Submit PR:**
   - Create PR to https://github.com/flathub/flathub
   - Reference your app repository
   - Wait for review

4. **Review Process:**
   - Flathub team reviews manifest
   - Tests build
   - Approves or requests changes

### Requirements for Flathub

✅ **Must Have:**
- [ ] Valid AppData XML (`app.pommersche.LazyLlama.metainfo.xml`)
- [ ] GPL-compatible license
- [ ] Source hosted on public repository (GitHub)
- [ ] Buildable manifest
- [ ] No proprietary dependencies

✅ **Recommended:**
- [ ] Desktop file (even for CLI apps)
- [ ] Icon (PNG, 128x128 minimum)
- [ ] Screenshot
- [ ] Detailed description

## 🧪 Testing

### Local Test

```bash
# Build and test
./scripts/build-flatpak.sh build
./scripts/build-flatpak.sh test

# Or install and run
./scripts/build-flatpak.sh install
flatpak run app.pommersche.LazyLlama
```

### Test Specific Version

```bash
# Update manifest to specific version
./scripts/build-flatpak.sh update

# Build
flatpak-builder --force-clean flatpak-build flatpak/app.pommersche.LazyLlama.json

# Run
flatpak-builder --run flatpak-build flatpak/app.pommersche.LazyLlama.json lazyllama
```

### Debug Build Issues

```bash
# Verbose build
flatpak-builder --verbose --force-clean flatpak-build flatpak/app.pommersche.LazyLlama.json

# Interactive shell
flatpak-builder --run flatpak-build flatpak/app.pommersche.LazyLlama.json bash
```

## 🔄 Update Workflow

When releasing new version:

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml

# 2. Commit and push to GitHub
git commit -am "Bump version to X.Y.Z"
git push

# 3. Create GitHub release
./scripts/release-github.sh

# 4. Update Flatpak
./scripts/build-flatpak.sh setup     # Regenerate deps
./scripts/build-flatpak.sh update    # Update manifest
./scripts/build-flatpak.sh build     # Test build

# 5. Deploy to Flathub
./scripts/deploy-flathub.sh update
# Then push and create PR
```

## 📝 Manifest Configuration

### Permissions Explained

```json
"finish-args": [
  "--share=network",        // Ollama API access
  "--share=ipc",           // IPC for clipboard
  "--socket=fallback-x11", // X11 clipboard fallback
  "--socket=wayland",      // Wayland clipboard
  "--device=dri",          // GPU access (optional)
  "--filesystem=host"      // Full filesystem (consider restricting)
]
```

### Restrict Filesystem

For production, consider:

```json
"--filesystem=xdg-config/lazyllama:create",  // Config only
"--filesystem=xdg-cache/lazyllama:create"    // Cache only
```

### Runtime Selection

**org.freedesktop.Platform** (current):
- Minimal base
- Best for CLI tools
- 23.08 = Latest stable

**Alternative: org.gnome.Platform**
- GNOME stack
- Use if switching to GTK UI

## 🐛 Troubleshooting

### "Failed to download sources"

**Cause:** `generated-sources.json` outdated or missing

**Fix:**
```bash
./scripts/build-flatpak.sh setup
```

### "Build failed: cargo not found"

**Cause:** Rust SDK extension not installed

**Fix:**
```bash
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08
```

### "Permission denied: network"

**Cause:** Ollama server unreachable from Flatpak

**Fix:** Ensure Ollama is running and accessible:
```bash
curl http://localhost:11434/api/tags
```

### "Manifest validation failed"

**Cause:** Invalid JSON or AppData XML

**Fix:**
```bash
# Validate JSON
jq . flatpak/app.pommersche.LazyLlama.json

# Validate AppData
appstreamcli validate flatpak/app.pommersche.LazyLlama.metainfo.xml
```

## 📚 Resources

- [Flatpak Documentation](https://docs.flatpak.org/)
- [Flathub Submission](https://github.com/flathub/flathub/wiki/App-Submission)
- [Flatpak Builder Tools](https://github.com/flatpak/flatpak-builder-tools)
- [AppData Guidelines](https://www.freedesktop.org/software/appstream/docs/chap-Metadata.html)

## 🎯 Next Steps

1. **Build locally** and test thoroughly
2. **Create GitHub account** on Flathub
3. **Submit initial PR** with manifest
4. **Respond to review** feedback
5. **Update regularly** with new releases

After approval, LazyLlama will be available to millions of Linux users via Flathub! 🎉
