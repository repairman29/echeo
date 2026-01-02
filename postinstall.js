#!/usr/bin/env node

/**
 * Echeo Post-Install Script
 * 
 * Verifies installation and provides helpful information
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const BIN_DIR = path.join(__dirname, 'bin');
const BIN_PATH = path.join(BIN_DIR, 'echeo');

function checkInstallation() {
  const binExists = fs.existsSync(BIN_PATH) || fs.existsSync(BIN_PATH + '.exe');
  
  if (!binExists) {
    console.warn('⚠️  Echeo binary not found. Run: npm run install');
    return false;
  }
  
  try {
    // Try to run echeo --version
    const version = execSync(`${BIN_PATH} --version`, { encoding: 'utf-8' });
    console.log('✅ Echeo installed successfully!');
    console.log(`   Version: ${version.trim()}`);
    console.log('\n📖 Quick start:');
    console.log('   echeo --path . --skip-embeddings --skip-summaries');
    console.log('   echeo --help');
    return true;
  } catch (error) {
    console.warn('⚠️  Echeo binary found but not executable');
    return false;
  }
}

if (require.main === module) {
  checkInstallation();
}

module.exports = { checkInstallation };

