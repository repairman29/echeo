#!/usr/bin/env node

/**
 * Echeo Install Script
 * 
 * This script handles installation of the Echeo binary.
 * It tries multiple methods:
 * 1. Download pre-built binary from GitHub Releases
 * 2. Build from source using cargo (if Rust is installed)
 * 3. Provide helpful error messages
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const https = require('https');
const os = require('os');

const VERSION = require('./package.json').version;
const BIN_DIR = path.join(__dirname, 'bin');
const BIN_PATH = path.join(BIN_DIR, 'echeo');

// Determine platform and architecture
function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();
  
  if (platform === 'darwin') {
    return arch === 'arm64' ? 'aarch64-apple-darwin' : 'x86_64-apple-darwin';
  } else if (platform === 'linux') {
    return arch === 'arm64' ? 'aarch64-unknown-linux-gnu' : 'x86_64-unknown-linux-gnu';
  } else if (platform === 'win32') {
    return arch === 'arm64' ? 'aarch64-pc-windows-msvc' : 'x86_64-pc-windows-msvc';
  }
  
  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

// Check if Rust/cargo is available
function hasRust() {
  try {
    execSync('cargo --version', { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

// Build from source
function buildFromSource() {
  console.log('🔨 Building Echeo from source...');
  try {
    execSync('cargo build --release', { 
      stdio: 'inherit',
      cwd: __dirname 
    });
    
    // Copy binary to bin directory
    const target = getPlatform();
    const sourcePath = path.join(__dirname, 'target', 'release', 'echeo');
    const destPath = path.join(BIN_DIR, 'echeo');
    
    if (process.platform === 'win32') {
      fs.copyFileSync(sourcePath + '.exe', destPath + '.exe');
    } else {
      fs.copyFileSync(sourcePath, destPath);
      fs.chmodSync(destPath, '755');
    }
    
    console.log('✅ Built successfully!');
    return true;
  } catch (error) {
    console.error('❌ Build failed:', error.message);
    return false;
  }
}

// Download pre-built binary
function downloadBinary() {
  const platform = getPlatform();
  const ext = process.platform === 'win32' ? '.exe' : '';
  const url = `https://github.com/yourusername/echeo/releases/download/v${VERSION}/echeo-${platform}${ext}`;
  
  console.log(`📥 Downloading Echeo for ${platform}...`);
  
  return new Promise((resolve, reject) => {
    https.get(url, (response) => {
      if (response.statusCode === 404) {
        reject(new Error('Pre-built binary not available for this platform'));
        return;
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Download failed: ${response.statusCode}`));
        return;
      }
      
      const filePath = path.join(BIN_DIR, `echeo${ext}`);
      const fileStream = fs.createWriteStream(filePath);
      
      response.pipe(fileStream);
      
      fileStream.on('finish', () => {
        if (process.platform !== 'win32') {
          fs.chmodSync(filePath, '755');
        }
        console.log('✅ Downloaded successfully!');
        resolve();
      });
      
      fileStream.on('error', reject);
    }).on('error', reject);
  });
}

// Main installation logic
async function install() {
  // Create bin directory
  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }
  
  // Check if binary already exists
  if (fs.existsSync(BIN_PATH) || fs.existsSync(BIN_PATH + '.exe')) {
    console.log('✅ Echeo binary already exists');
    return;
  }
  
  console.log('🚀 Installing Echeo...');
  
  // Try to download pre-built binary first
  try {
    await downloadBinary();
    return;
  } catch (error) {
    console.log(`⚠️  Download failed: ${error.message}`);
  }
  
  // Fall back to building from source
  if (hasRust()) {
    if (buildFromSource()) {
      return;
    }
  }
  
  // If all else fails, provide helpful error
  console.error('\n❌ Installation failed!');
  console.error('\nOptions:');
  console.error('1. Install Rust: https://rustup.rs/');
  console.error('2. Download binary manually from: https://github.com/yourusername/echeo/releases');
  console.error('3. Use pre-built binaries from GitHub Releases');
  
  process.exit(1);
}

// Run installation
if (require.main === module) {
  install().catch((error) => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { install };

