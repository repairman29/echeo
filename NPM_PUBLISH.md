# Publishing Echeo to npm

## 🚀 Quick Publish Guide

### Step 1: Prepare for Publishing

```bash
cd payload-cli

# Make sure you're logged into npm
npm login

# Check your npm account
npm whoami
```

### Step 2: Build Binaries (Optional)

If you want to include pre-built binaries in the package:

```bash
# Build for your platform
cargo build --release

# Copy to bin directory
mkdir -p bin
cp target/release/echeo bin/echeo
chmod +x bin/echeo
```

### Step 3: Test Locally

```bash
# Test the package locally
npm pack

# Install it locally to test
npm install -g ./echeo-0.1.0.tgz

# Test it works
echeo --version
```

### Step 4: Publish to npm

```bash
# Dry run first (see what will be published)
npm publish --dry-run

# Actually publish
npm publish
```

---

## 📋 Pre-Publish Checklist

- [ ] Update version in `package.json`
- [ ] Update version in `Cargo.toml` (should match)
- [ ] Test installation: `npm install -g echeo`
- [ ] Test binary works: `echeo --version`
- [ ] Update README with npm install instructions
- [ ] Check `.npmignore` excludes unnecessary files

---

## 🔧 Configuration

### package.json Fields

- **name**: `echeo` (must be unique on npm)
- **version**: Follows semver (0.1.0)
- **bin**: Points to the binary location
- **files**: What gets included in the package

### Binary Location

The install script will:
1. Try to download pre-built binary from GitHub Releases
2. Fall back to building from source if Rust is installed
3. Provide helpful error messages if both fail

---

## 📦 What Gets Published

The `.npmignore` file controls what gets included:

**Included:**
- `bin/` - Binary directory
- `install.js` - Installation script
- `postinstall.js` - Post-install verification
- `README.md` - Documentation
- `package.json` - Package metadata

**Excluded:**
- `src/` - Rust source code
- `target/` - Build artifacts
- `Cargo.toml` - Rust config (unless needed)
- Documentation files (except README)

---

## 🎯 Installation Methods

Users can install Echeo via npm in several ways:

### Method 1: Global Install
```bash
npm install -g echeo
echeo --version
```

### Method 2: Local Install
```bash
npm install echeo
npx echeo --version
```

### Method 3: As Dev Dependency
```bash
npm install --save-dev echeo
npx echeo --version
```

---

## 🔄 Updating the Package

### Version Bump

```bash
# Patch version (0.1.0 -> 0.1.1)
npm version patch

# Minor version (0.1.0 -> 0.2.0)
npm version minor

# Major version (0.1.0 -> 1.0.0)
npm version major

# Then publish
npm publish
```

### After Publishing

1. Update GitHub Releases with new binaries
2. Update documentation
3. Announce on social media/community

---

## 🐛 Troubleshooting

### "Package name already taken"
**Fix:** Choose a different name or use scoped package:
```json
{
  "name": "@yourusername/echeo"
}
```

### "Binary not found after install"
**Fix:** Make sure `bin/echeo` exists and is executable:
```bash
chmod +x bin/echeo
```

### "Install script fails"
**Fix:** Check that install.js has proper permissions:
```bash
chmod +x install.js
```

---

## 📊 Publishing Strategy

### Option 1: Source-Only Package
- Publish package.json + install script
- Users build from source (requires Rust)
- Smaller package size
- **Recommended for now**

### Option 2: Pre-Built Binaries
- Include binaries for all platforms
- Larger package size
- Faster installation
- Requires GitHub Releases setup

### Option 3: Hybrid (Recommended)
- Package includes install script
- Script tries to download binary first
- Falls back to building from source
- Best user experience

---

## 🎯 Next Steps

1. **Test locally**: `npm pack` and `npm install -g ./echeo-0.1.0.tgz`
2. **Publish**: `npm publish`
3. **Verify**: `npm install -g echeo && echeo --version`
4. **Update docs**: Add npm install instructions to README

---

**Ready to publish!** 🚀

