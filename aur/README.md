# AUR Packages for LazyLlama

This directory contains PKGBUILDs for Arch Linux AUR (Arch User Repository).

## Packages

### lazyllama
Source-based package that builds LazyLlama from the crates.io source.

**Advantages:**
- Works on all architectures (x86_64, aarch64, i686, armv7h)
- Always uses the latest optimizations for your system
- Verifiable build process

**Disadvantages:**
- Longer installation time (needs to compile)
- Requires Rust toolchain

### lazyllama-bin
Binary package that installs pre-built LazyLlama from GitHub releases.

**Advantages:**
- Fast installation (no compilation needed)
- No build dependencies required

**Disadvantages:**
- Only x86_64 architecture
- Slightly larger package size

## Quick Start

### First-time AUR Setup

1. **Clone the AUR repositories:**
```bash
cd aur/lazyllama
git clone ssh://aur@aur.archlinux.org/lazyllama.git aur-repo

cd ../lazyllama-bin
git clone ssh://aur@aur.archlinux.org/lazyllama-bin.git aur-repo
```

2. **Configure your AUR SSH key** (if not already done):
```bash
# Generate SSH key if needed
ssh-keygen -t ed25519 -C "your-email@example.com"

# Add to AUR account at https://aur.archlinux.org/account/
cat ~/.ssh/id_ed25519.pub
```

## Deployment Workflow

### 1. Update Version in Cargo.toml
```bash
# Edit version in Cargo.toml
vim Cargo.toml
```

### 2. Publish to crates.io
```bash
cargo publish
```

### 3. Create GitHub Release
```bash
# Build release binary
cargo build --release

# Create tarball
tar -czf lazyllama-0.4.1-x86_64.tar.gz -C target/release lazyllama

# Create GitHub release with tag v0.4.1
gh release create v0.4.1 lazyllama-0.4.1-x86_64.tar.gz \
    --title "v0.4.1" \
    --notes "Release notes here"
```

### 4. Update and Deploy AUR Packages

**Update PKGBUILDs (without pushing):**
```bash
./scripts/deploy-aur.sh
```

**Update and push to AUR:**
```bash
./scripts/deploy-aur.sh --push
```

**Update only specific package:**
```bash
./scripts/deploy-aur.sh --package lazyllama
./scripts/deploy-aur.sh --package lazyllama-bin --push
```

## Manual Deployment

If you prefer manual control:

### For lazyllama (source)
```bash
cd aur/lazyllama

# Update PKGBUILD version
vim PKGBUILD

# Calculate checksum from crates.io
curl -sL https://static.crates.io/crates/lazyllama/lazyllama-0.4.1.crate | sha256sum

# Update sha256sums in PKGBUILD
vim PKGBUILD

# Test build
makepkg --printsrcinfo > .SRCINFO

# Push to AUR
cd aur-repo
cp ../PKGBUILD ../. SRCINFO .
git add PKGBUILD .SRCINFO
git commit -m "Update to version 0.4.1"
git push
```

### For lazyllama-bin (binary)
```bash
cd aur/lazyllama-bin

# Update PKGBUILD version
vim PKGBUILD

# Calculate checksum from GitHub release
curl -sL https://github.com/Pommersche92/lazyllama/releases/download/v0.4.1/lazyllama-0.4.1-x86_64.tar.gz | sha256sum

# Update sha256sums in PKGBUILD
vim PKGBUILD

# Test build
makepkg --printsrcinfo > .SRCINFO

# Push to AUR
cd aur-repo
cp ../PKGBUILD ./.SRCINFO .
git add PKGBUILD .SRCINFO
git commit -m "Update to version 0.4.1"
git push
```

## Troubleshooting

### "Failed to download crate from crates.io"
- Make sure you've run `cargo publish` first
- Wait a few minutes for crates.io to sync

### "Failed to download release from GitHub"
- Ensure the GitHub release exists with tag `v0.4.1`
- Verify the tarball name matches: `lazyllama-0.4.1-x86_64.tar.gz`

### "PKGBUILD validation failed"
- Check syntax errors in PKGBUILD
- Run `makepkg --printsrcinfo` manually to see errors

### "Permission denied (publickey)"
- Verify your SSH key is added to your AUR account
- Test connection: `ssh aur@aur.archlinux.org`

## Testing Locally

Test build before pushing:

```bash
cd aur/lazyllama
makepkg -si  # Build and install

cd ../lazyllama-bin
makepkg -si  # Build and install
```

## Best Practices

1. **Always test locally** before pushing to AUR
2. **Update both packages** simultaneously to keep versions in sync
3. **Write meaningful commit messages** describing changes
4. **Increment pkgrel** if updating PKGBUILD without version bump
5. **Keep .SRCINFO in sync** with PKGBUILD using `makepkg --printsrcinfo`

## Resources

- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [PKGBUILD Documentation](https://wiki.archlinux.org/title/PKGBUILD)
- [cargo-aur Documentation](https://crates.io/crates/cargo-aur)
