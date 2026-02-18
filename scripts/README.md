# Release Scripts

Automation scripts for releasing LazyLlama to multiple platforms.

## 📋 Overview

Three scripts handle the complete release pipeline:

1. **`release.sh`** - Complete pipeline (recommended)
2. **`release-github.sh`** - GitHub release only
3. **`deploy-aur.sh`** - AUR deployment only

## 🚀 Quick Start

### Complete Release (Recommended)

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml

# 2. Commit changes
git add Cargo.toml
git commit -m "Bump version to 0.4.2"
git push

# 3. Run complete release pipeline
./scripts/release.sh
```

This will:
- ✅ Run tests
- ✅ Publish to crates.io
- ✅ Build release binary
- ✅ Create GitHub release with tarball
- ✅ Deploy to AUR (lazyllama + lazyllama-bin)

### Individual Steps

If you need more control:

```bash
# 1. Publish to crates.io
cargo publish

# 2. Create GitHub release
./scripts/release-github.sh

# 3. Deploy to AUR
./scripts/deploy-aur.sh --push
```

## 📖 Script Details

### release.sh - Complete Pipeline

**Complete automated release workflow.**

```bash
./scripts/release.sh              # Full pipeline with prompts
./scripts/release.sh --draft      # Create draft GitHub release
./scripts/release.sh --skip-crates # Skip crates.io (already published)
./scripts/release.sh --skip-github # Skip GitHub release
./scripts/release.sh --skip-aur   # Skip AUR deployment
```

**What it does:**
1. Runs `cargo test --all-features`
2. Publishes to crates.io (with confirmation)
3. Builds release binary
4. Creates tarball with binary, LICENSE, and README
5. Creates GitHub release
6. Updates and pushes AUR packages (with confirmation)

**Requirements:**
- Logged in to cargo: `cargo login`
- GitHub CLI authenticated: `gh auth login`
- AUR repositories cloned (see `aur/README.md`)

---

### release-github.sh - GitHub Release

**Creates GitHub release with pre-built binary tarball.**

```bash
./scripts/release-github.sh                                    # Normal release
./scripts/release-github.sh --draft                           # Draft release
./scripts/release-github.sh --notes "Bug fixes"               # With notes
./scripts/release-github.sh --skip-build                      # Use existing binary
```

**What it does:**
1. Reads version from `Cargo.toml`
2. Builds release binary (`cargo build --release`)
3. Creates tarball: `lazyllama-VERSION-x86_64.tar.gz`
   - Contains: binary, LICENSE, README.md
4. Creates GitHub release with tag `vVERSION`
5. Uploads tarball as release asset

**Output:**
- Tarball in `target/dist/`
- GitHub release at `https://github.com/Pommersche92/lazyllama/releases`

**Requirements:**
- GitHub CLI: `sudo pacman -S github-cli`
- Authenticated: `gh auth login`

---

### deploy-aur.sh - AUR Deployment

**Updates and deploys both AUR packages.**

```bash
./scripts/deploy-aur.sh                           # Update PKGBUILDs (dry-run)
./scripts/deploy-aur.sh --push                    # Update and push to AUR
./scripts/deploy-aur.sh --package lazyllama       # Only lazyllama
./scripts/deploy-aur.sh --package lazyllama-bin --push  # Only lazyllama-bin
```

**What it does:**
1. Reads version from `Cargo.toml`
2. Updates `pkgver=` and `pkgrel=1` in PKGBUILDs
3. Downloads and calculates SHA256 checksums:
   - `lazyllama`: from crates.io
   - `lazyllama-bin`: from GitHub releases
4. Validates PKGBUILDs with `makepkg --printsrcinfo`
5. Commits and pushes to AUR (with `--push`)

**Requirements:**
- AUR SSH key configured
- AUR repos cloned in `aur/*/aur-repo/`

**First-time setup:** See `aur/README.md`

## 🔧 Initial Setup

### 1. Cargo (crates.io)

```bash
# Get API token from https://crates.io/settings/tokens
cargo login
```

### 2. GitHub CLI

```bash
# Install
sudo pacman -S github-cli

# Authenticate
gh auth login
```

### 3. AUR

```bash
# Generate SSH key
ssh-keygen -t ed25519 -C "your-email@example.com"

# Add to AUR account: https://aur.archlinux.org/account/
cat ~/.ssh/id_ed25519.pub

# Clone AUR repositories
cd aur/lazyllama
git clone ssh://aur@aur.archlinux.org/lazyllama.git aur-repo

cd ../lazyllama-bin
git clone ssh://aur@aur.archlinux.org/lazyllama-bin.git aur-repo
```

## 📝 Release Checklist

- [ ] Update `CHANGELOG.md` (if exists)
- [ ] Update version in `Cargo.toml`
- [ ] Update version in `Cargo.lock`: `cargo update`
- [ ] Run tests: `cargo test --all-features`
- [ ] Commit changes: `git commit -am "Bump version to X.Y.Z"`
- [ ] Push to GitHub: `git push`
- [ ] Run release script: `./scripts/release.sh`
- [ ] Verify releases:
  - [ ] https://crates.io/crates/lazyllama
  - [ ] https://github.com/Pommersche92/lazyllama/releases
  - [ ] https://aur.archlinux.org/packages/lazyllama
  - [ ] https://aur.archlinux.org/packages/lazyllama-bin

## 🐛 Troubleshooting

### "Failed to download crate from crates.io"
**Cause:** Version not published yet or crates.io sync delay

**Solution:**
```bash
# Publish manually first
cargo publish

# Wait a few minutes, then retry
./scripts/deploy-aur.sh --package lazyllama --push
```

### "Failed to download release from GitHub"
**Cause:** GitHub release doesn't exist or wrong tarball name

**Solution:**
```bash
# Create GitHub release first
./scripts/release-github.sh

# Then deploy AUR
./scripts/deploy-aur.sh --package lazyllama-bin --push
```

### "Release vX.Y.Z already exists"
**Cause:** You're trying to release the same version twice

**Solution:**
```bash
# Option 1: Delete existing release
gh release delete vX.Y.Z

# Option 2: Bump version in Cargo.toml
vim Cargo.toml
```

### "PKGBUILD validation failed"
**Cause:** Syntax error in PKGBUILD

**Solution:**
```bash
cd aur/lazyllama  # or lazyllama-bin
makepkg --printsrcinfo  # See error message
vim PKGBUILD  # Fix issue
```

### "Permission denied (publickey)" (AUR)
**Cause:** SSH key not configured for AUR

**Solution:**
```bash
# Test connection
ssh -T aur@aur.archlinux.org

# Add key to AUR if failed
cat ~/.ssh/id_ed25519.pub
# Paste at https://aur.archlinux.org/account/
```

## 🎯 Common Workflows

### Full Release (Normal)

```bash
# 1. Update version
vim Cargo.toml

# 2. Commit
git add Cargo.toml
git commit -m "Release v0.4.2"
git push

# 3. Release everything
./scripts/release.sh
```

### Hotfix Release

```bash
# 1. Fix bug and bump version
vim src/lib.rs Cargo.toml
git commit -am "Fix critical bug - v0.4.2"
git push

# 2. Quick release
./scripts/release.sh
```

### Re-release AUR Only

```bash
# If crates.io and GitHub already done
./scripts/deploy-aur.sh --push
```

### Create Draft Release

```bash
# For testing
./scripts/release-github.sh --draft

# Review at https://github.com/Pommersche92/lazyllama/releases
# Publish manually when ready
```

### Update Only One AUR Package

```bash
# Update only source package
./scripts/deploy-aur.sh --package lazyllama --push

# Update only binary package
./scripts/deploy-aur.sh --package lazyllama-bin --push
```

## 📚 Resources

- [Cargo Publish Documentation](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub CLI Manual](https://cli.github.com/manual/)
- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [PKGBUILD Reference](https://wiki.archlinux.org/title/PKGBUILD)
