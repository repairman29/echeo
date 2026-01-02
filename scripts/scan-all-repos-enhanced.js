#!/usr/bin/env node

/**
 * Enhanced Complete Repository Catalog
 * 
 * Uses comprehensive-repo-audit.js to get ALL repos (public + private)
 * Then uses Echeo to extract capabilities
 * Parses Echeo output even without embeddings
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const { createClient } = require('@supabase/supabase-js');

const SUPABASE_URL = process.env.SUPABASE_URL || 'https://rbfzlqmkwhbvrrfdcain.supabase.co';
const SUPABASE_SERVICE_ROLE_KEY = process.env.SUPABASE_SERVICE_ROLE_KEY || 'sb_secret_Ct-MkMyeSNyQo7RST6gCvw_j8u3_gIH';
const supabase = createClient(SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY);

let Octokit;
try {
  Octokit = require('@octokit/rest').Octokit;
} catch (e) {
  console.error('❌ @octokit/rest not found. Run: npm install @octokit/rest');
  process.exit(1);
}

async function getGitHubToken() {
  if (process.env.GITHUB_TOKEN) return process.env.GITHUB_TOKEN;
  
  try {
    const { data } = await supabase
      .from('app_config')
      .select('value')
      .eq('key', 'github_token')
      .maybeSingle();
    return data?.value?.token || null;
  } catch (e) {
    return null;
  }
}

async function getAllRepos(octokit) {
  const repos = [];
  let page = 1;
  let hasMore = true;

  while (hasMore) {
    try {
      const { data } = await octokit.repos.listForUser({
        username: 'repairman29',
        type: 'all', // This gets both public AND private
        per_page: 100,
        page: page
      });

      if (data.length === 0) {
        hasMore = false;
      } else {
        repos.push(...data.map(r => ({
          full_name: r.full_name,
          name: r.name,
          private: r.private,
          description: r.description,
          language: r.language,
          stars: r.stargazers_count,
          forks: r.forks_count,
          updated_at: r.updated_at,
          pushed_at: r.pushed_at,
          archived: r.archived
        })));
        page++;
        if (data.length < 100) hasMore = false;
      }
    } catch (error) {
      console.error('❌ Error fetching repos:', error.message);
      hasMore = false;
    }
  }

  return repos;
}

function parseEcheoOutput(stdout, stderr) {
  const capabilities = {
    filesScanned: 0,
    signals: 0,
    capabilities: 0,
    details: []
  };

  // Parse the output
  const filesMatch = stdout.match(/(\d+)\s+Files Scanned/i);
  if (filesMatch) {
    capabilities.filesScanned = parseInt(filesMatch[1]);
  }

  const signalsMatch = stdout.match(/CONTACTS FOUND:\s+(\d+)\s+VALID SIGNALS with\s+(\d+)\s+CAPABILITIES/i);
  if (signalsMatch) {
    capabilities.signals = parseInt(signalsMatch[1]);
    capabilities.capabilities = parseInt(signalsMatch[2]);
  }

  // Try to extract capability names from TypeScript/JavaScript parsing
  const typeScriptMatches = stdout.match(/\[TYPESCRIPT\]\s+(.+)/g);
  if (typeScriptMatches) {
    capabilities.details = typeScriptMatches.map(match => {
      const file = match.replace(/\[TYPESCRIPT\]\s+/, '');
      return {
        file,
        type: 'typescript',
        extracted: true
      };
    });
  }

  return capabilities;
}

function scanRepoWithEcheo(repo, githubToken) {
  return new Promise((resolve) => {
    const repoName = repo.split('/')[1];
    const outputDir = path.join(__dirname, '../docs/repo-catalog');
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    const echeoProcess = spawn('echeo', [
      '--github-token', githubToken,
      '--github-repo', repo,
      '--skip-embeddings',
      '--skip-summaries',
      '--generate-loadout'
    ], {
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let stdout = '';
    let stderr = '';

    echeoProcess.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    echeoProcess.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    echeoProcess.on('close', (code) => {
      // Parse output even without loadout.json
      const parsed = parseEcheoOutput(stdout, stderr);
      
      // Also check for loadout.json
      const loadoutPath = path.join(process.cwd(), 'loadout.json');
      if (fs.existsSync(loadoutPath)) {
        try {
          const loadout = JSON.parse(fs.readFileSync(loadoutPath, 'utf8'));
          parsed.capabilityData = loadout.capabilities || [];
          parsed.capabilities = parsed.capabilityData.length;
          fs.unlinkSync(loadoutPath);
        } catch (e) {
          // Ignore
        }
      }

      // Save log
      const logFile = path.join(outputDir, `${repoName}.log`);
      fs.writeFileSync(logFile, `STDOUT:\n${stdout}\n\nSTDERR:\n${stderr}\n\nEXIT CODE: ${code}`);

      resolve({
        repo,
        capabilities: parsed.capabilities,
        signals: parsed.signals,
        filesScanned: parsed.filesScanned,
        capabilityData: parsed.capabilityData || [],
        details: parsed.details,
        success: parsed.capabilities > 0 || parsed.signals > 0,
        stdout,
        stderr
      });
    });
  });
}

async function main() {
  console.log('🚀 Starting Enhanced Complete Repository Catalog Scan...\n');

  const githubToken = await getGitHubToken();
  if (!githubToken) {
    console.error('❌ No GitHub token found.');
    process.exit(1);
  }

  const octokit = new Octokit({ auth: githubToken });

  // Get ALL repos (public + private)
  console.log('📋 Fetching ALL repositories (public + private)...');
  const repos = await getAllRepos(octokit);
  console.log(`✅ Found ${repos.length} repositories (${repos.filter(r => r.private).length} private)\n`);

  // Check Echeo CLI
  try {
    execSync('which echeo', { stdio: 'ignore' });
  } catch (e) {
    console.error('❌ Echeo CLI not found. Install with: npm install -g echeo');
    process.exit(1);
  }

  console.log('🔍 Scanning all repositories with Echeo...\n');
  console.log('⚠️  This will take a while (2+ seconds per repo)...\n');

  const allResults = [];
  let totalCapabilities = 0;
  let totalSignals = 0;
  let totalFiles = 0;

  for (let i = 0; i < repos.length; i++) {
    const repo = repos[i];
    const label = `${repo.full_name}${repo.private ? ' 🔒' : ''}${repo.archived ? ' 📦' : ''}`;
    console.log(`[${i + 1}/${repos.length}] ${label}...`);

    const result = await scanRepoWithEcheo(repo.full_name, githubToken);
    
    allResults.push({
      ...repo,
      ...result
    });

    totalCapabilities += result.capabilities;
    totalSignals += result.signals;
    totalFiles += result.filesScanned;

    if (result.success) {
      console.log(`   ✅ ${result.capabilities} capabilities, ${result.signals} signals, ${result.filesScanned} files`);
    } else {
      console.log(`   ⚠️  ${result.capabilities} capabilities (may need embeddings or repo may be empty)`);
    }

    // Rate limiting
    if (i < repos.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }

  console.log('\n📊 Generating comprehensive catalog...\n');

  const outputDir = path.join(__dirname, '../docs/repo-catalog');
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Sort by capabilities
  const sortedResults = [...allResults].sort((a, b) => b.capabilities - a.capabilities);

  // Generate catalog
  let catalog = `# Complete Repository Capability Catalog
## The Pantry - All Your Ingredients

**Date:** ${new Date().toISOString().split('T')[0]}  
**Status:** ✅ **Complete Inventory**

---

## 📊 Executive Summary

**Total Repositories:** ${repos.length}  
**Public:** ${repos.filter(r => !r.private).length} | **Private:** ${repos.filter(r => r.private).length}  
**Archived:** ${repos.filter(r => r.archived).length} | **Active:** ${repos.filter(r => !r.archived).length}

**Total Capabilities Found:** ${totalCapabilities}  
**Total Signals:** ${totalSignals}  
**Total Files Scanned:** ${totalFiles.toLocaleString()}  
**Average Capabilities per Repo:** ${(totalCapabilities / repos.length).toFixed(1)}

---

## 🎯 Top Repositories by Capabilities

`;

  sortedResults.slice(0, 20).forEach((result, index) => {
    catalog += `### ${index + 1}. ${result.full_name}${result.private ? ' 🔒' : ''}${result.archived ? ' 📦' : ''}\n\n`;
    catalog += `- **Capabilities:** ${result.capabilities}\n`;
    catalog += `- **Signals:** ${result.signals}\n`;
    catalog += `- **Files Scanned:** ${result.filesScanned.toLocaleString()}\n`;
    catalog += `- **Language:** ${result.language || 'N/A'}\n`;
    catalog += `- **Description:** ${result.description || 'No description'}\n`;
    catalog += `- **Stars:** ${result.stars || 0} | **Forks:** ${result.forks || 0}\n`;
    catalog += `- **Updated:** ${new Date(result.updated_at).toLocaleDateString()}\n\n`;
    
    if (result.details && result.details.length > 0) {
      catalog += `**Files with Capabilities:**\n`;
      result.details.slice(0, 10).forEach(detail => {
        catalog += `- ${detail.file}\n`;
      });
      if (result.details.length > 10) {
        catalog += `- ... and ${result.details.length - 10} more\n`;
      }
      catalog += `\n`;
    }
    
    catalog += `---\n\n`;
  });

  catalog += `## 📦 Complete Repository List\n\n`;
  sortedResults.forEach((result) => {
    const status = result.private ? '🔒' : '🌐';
    const archived = result.archived ? '📦' : '';
    catalog += `- **${status} ${archived} ${result.full_name}**: ${result.capabilities} capabilities, ${result.signals} signals, ${result.filesScanned} files\n`;
  });

  catalog += `\n---\n\n`;
  catalog += `**Last Updated:** ${new Date().toISOString()}\n`;
  catalog += `**Generated By:** Enhanced Repository Catalog Scanner\n`;

  fs.writeFileSync(path.join(outputDir, 'COMPLETE_CATALOG.md'), catalog);

  // JSON Export
  fs.writeFileSync(
    path.join(outputDir, 'COMPLETE_CATALOG.json'),
    JSON.stringify({
      summary: {
        totalRepos: repos.length,
        publicRepos: repos.filter(r => !r.private).length,
        privateRepos: repos.filter(r => r.private).length,
        archivedRepos: repos.filter(r => r.archived).length,
        totalCapabilities,
        totalSignals,
        totalFiles,
        averageCapabilities: (totalCapabilities / repos.length).toFixed(1)
      },
      repos: allResults
    }, null, 2)
  );

  console.log('✅ Enhanced catalog complete!');
  console.log(`   Complete Catalog: ${path.join(outputDir, 'COMPLETE_CATALOG.md')}`);
  console.log(`   JSON Export: ${path.join(outputDir, 'COMPLETE_CATALOG.json')}\n`);

  console.log('📊 Summary:');
  console.log(`   Total Repos: ${repos.length} (${repos.filter(r => r.private).length} private)`);
  console.log(`   Total Capabilities: ${totalCapabilities}`);
  console.log(`   Total Signals: ${totalSignals}`);
  console.log(`   Total Files: ${totalFiles.toLocaleString()}`);
  console.log(`   Top Repo: ${sortedResults[0].full_name} (${sortedResults[0].capabilities} capabilities)\n`);

  console.log('🍳 Your pantry is ready! Check the catalog to see all your ingredients!\n');
}

main().catch(console.error);

