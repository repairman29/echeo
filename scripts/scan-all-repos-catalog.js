#!/usr/bin/env node

/**
 * Complete Repository Catalog
 * 
 * Scans ALL repos and creates:
 * 1. Complete capability inventory (the "pantry")
 * 2. Redundancy detection (duplicate capabilities)
 * 3. Cross-repo capability mapping
 * 4. Product combination opportunities
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const { createClient } = require('@supabase/supabase-js');

// Supabase config
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
        type: 'all',
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
          pushed_at: r.pushed_at
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
      const loadoutPath = path.join(process.cwd(), 'loadout.json');
      let capabilities = [];
      
      if (fs.existsSync(loadoutPath)) {
        try {
          const loadout = JSON.parse(fs.readFileSync(loadoutPath, 'utf8'));
          capabilities = loadout.capabilities || [];
          fs.unlinkSync(loadoutPath);
        } catch (e) {
          // Ignore parse errors
        }
      }

      // Also try to extract from stdout (capabilities are logged)
      const capabilityMatches = stdout.match(/CONTACTS FOUND: (\d+) VALID SIGNALS with (\d+) CAPABILITIES/g);
      if (capabilityMatches) {
        const match = stdout.match(/CONTACTS FOUND: (\d+) VALID SIGNALS with (\d+) CAPABILITIES/);
        if (match) {
          const signalCount = parseInt(match[1]);
          const capabilityCount = parseInt(match[2]);
          // Store this info even if we don't have full capability details
        }
      }

      resolve({
        repo,
        capabilities: capabilities.length,
        capabilityData: capabilities,
        success: code === 0,
        stdout,
        stderr
      });
    });
  });
}

function findRedundancies(allCapabilities) {
  const capabilityMap = new Map(); // name -> [repos]
  const duplicates = [];

  allCapabilities.forEach(({ repo, capabilities }) => {
    capabilities.forEach(cap => {
      const name = cap.name || cap.capability_name || 'unknown';
      const normalized = name.toLowerCase().trim();
      
      if (!capabilityMap.has(normalized)) {
        capabilityMap.set(normalized, []);
      }
      capabilityMap.get(normalized).push({
        repo,
        capability: cap
      });
    });
  });

  // Find duplicates (same capability in multiple repos)
  capabilityMap.forEach((repos, name) => {
    if (repos.length > 1) {
      duplicates.push({
        name,
        repos: repos.map(r => r.repo),
        count: repos.length
      });
    }
  });

  return duplicates.sort((a, b) => b.count - a.count);
}

function createPantryCatalog(allCapabilities) {
  const pantry = {
    byCategory: {},
    byLanguage: {},
    byKind: {},
    all: []
  };

  allCapabilities.forEach(({ repo, capabilities }) => {
    capabilities.forEach(cap => {
      const entry = {
        name: cap.name || cap.capability_name || 'unknown',
        kind: cap.kind || 'unknown',
        language: cap.language || 'unknown',
        repo,
        code_snippet: cap.code_snippet || ''
      };

      pantry.all.push(entry);

      // Categorize
      if (!pantry.byKind[entry.kind]) {
        pantry.byKind[entry.kind] = [];
      }
      pantry.byKind[entry.kind].push(entry);

      if (!pantry.byLanguage[entry.language]) {
        pantry.byLanguage[entry.language] = [];
      }
      pantry.byLanguage[entry.language].push(entry);
    });
  });

  return pantry;
}

function findProductCombinations(pantry) {
  const combinations = [];

  // Find complementary capabilities that could be combined
  const apiRoutes = pantry.byKind['api_route'] || [];
  const services = pantry.byKind['class'] || [];
  const functions = pantry.byKind['function'] || [];

  // Example: API route + Service + Function = Complete Product
  apiRoutes.forEach(api => {
    const relatedServices = services.filter(s => 
      s.name.toLowerCase().includes(api.name.toLowerCase().split('/').pop()) ||
      api.name.toLowerCase().includes(s.name.toLowerCase())
    );

    if (relatedServices.length > 0) {
      combinations.push({
        type: 'API + Service',
        api,
        services: relatedServices,
        potentialProduct: `${api.name} Service`
      });
    }
  });

  return combinations;
}

async function main() {
  console.log('🚀 Starting Complete Repository Catalog Scan...\n');

  const githubToken = await getGitHubToken();
  if (!githubToken) {
    console.error('❌ No GitHub token found.');
    process.exit(1);
  }

  const octokit = new Octokit({ auth: githubToken });

  // Get all repos
  console.log('📋 Fetching all repositories...');
  const repos = await getAllRepos(octokit);
  console.log(`✅ Found ${repos.length} repositories\n`);

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
  const allCapabilities = [];

  for (let i = 0; i < repos.length; i++) {
    const repo = repos[i];
    console.log(`[${i + 1}/${repos.length}] ${repo.full_name}${repo.private ? ' (private)' : ''}...`);

    const result = await scanRepoWithEcheo(repo.full_name, githubToken);
    allResults.push({
      ...repo,
      ...result
    });

    if (result.capabilityData.length > 0) {
      allCapabilities.push({
        repo: repo.full_name,
        capabilities: result.capabilityData
      });
      console.log(`   ✅ ${result.capabilities} capabilities`);
    } else {
      console.log(`   ⚠️  ${result.capabilities} capabilities (may need embeddings)`);
    }

    // Rate limiting
    if (i < repos.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }

  console.log('\n📊 Analyzing results...\n');

  // Find redundancies
  const redundancies = findRedundancies(allCapabilities);
  console.log(`🔍 Found ${redundancies.length} duplicate capabilities across repos\n`);

  // Create pantry
  const pantry = createPantryCatalog(allCapabilities);
  console.log(`📦 Pantry created: ${pantry.all.length} total capabilities\n`);

  // Find combinations
  const combinations = findProductCombinations(pantry);
  console.log(`🔗 Found ${combinations.length} potential product combinations\n`);

  // Generate reports
  const outputDir = path.join(__dirname, '../docs/repo-catalog');
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // 1. Complete Catalog
  let catalog = `# Complete Repository Capability Catalog
## The Pantry - All Your Ingredients

**Date:** ${new Date().toISOString().split('T')[0]}  
**Status:** ✅ **Complete Inventory**

---

## 📊 Executive Summary

**Total Repositories:** ${repos.length}  
**Total Capabilities:** ${pantry.all.length}  
**Duplicate Capabilities:** ${redundancies.length}  
**Unique Capabilities:** ${pantry.all.length - redundancies.reduce((sum, d) => sum + d.count - 1, 0)}  
**Potential Product Combinations:** ${combinations.length}

---

## 🍳 The Pantry (All Capabilities)

### By Kind

`;

  Object.keys(pantry.byKind).forEach(kind => {
    catalog += `#### ${kind} (${pantry.byKind[kind].length})\n\n`;
    pantry.byKind[kind].slice(0, 20).forEach(cap => {
      catalog += `- **${cap.name}** (${cap.language}) - from ${cap.repo}\n`;
    });
    if (pantry.byKind[kind].length > 20) {
      catalog += `- ... and ${pantry.byKind[kind].length - 20} more\n`;
    }
    catalog += `\n`;
  });

  catalog += `### By Language\n\n`;
  Object.keys(pantry.byLanguage).forEach(lang => {
    catalog += `- **${lang}**: ${pantry.byLanguage[lang].length} capabilities\n`;
  });

  catalog += `\n---\n\n## 🔄 Redundancies (Duplicate Capabilities)\n\n`;
  catalog += `**Found ${redundancies.length} capabilities that exist in multiple repos:**\n\n`;

  redundancies.slice(0, 50).forEach(dup => {
    catalog += `### ${dup.name}\n\n`;
    catalog += `- **Found in ${dup.count} repos:**\n`;
    dup.repos.forEach(repo => {
      catalog += `  - ${repo}\n`;
    });
    catalog += `\n`;
  });

  catalog += `\n---\n\n## 🔗 Product Combination Opportunities\n\n`;
  combinations.slice(0, 20).forEach(combo => {
    catalog += `### ${combo.potentialProduct}\n\n`;
    catalog += `- **API:** ${combo.api.name} (${combo.api.repo})\n`;
    combo.services.forEach(service => {
      catalog += `- **Service:** ${service.name} (${service.repo})\n`;
    });
    catalog += `\n`;
  });

  catalog += `\n---\n\n## 📦 Complete Repository List\n\n`;
  allResults.forEach(result => {
    catalog += `### ${result.full_name}${result.private ? ' 🔒' : ''}\n\n`;
    catalog += `- **Description:** ${result.description || 'No description'}\n`;
    catalog += `- **Language:** ${result.language || 'N/A'}\n`;
    catalog += `- **Capabilities:** ${result.capabilities}\n`;
    catalog += `- **Stars:** ${result.stars || 0} | **Forks:** ${result.forks || 0}\n`;
    catalog += `- **Updated:** ${new Date(result.updated_at).toLocaleDateString()}\n\n`;
  });

  fs.writeFileSync(path.join(outputDir, 'COMPLETE_CATALOG.md'), catalog);

  // 2. JSON Export
  fs.writeFileSync(
    path.join(outputDir, 'COMPLETE_CATALOG.json'),
    JSON.stringify({
      repos: allResults,
      pantry,
      redundancies,
      combinations,
      summary: {
        totalRepos: repos.length,
        totalCapabilities: pantry.all.length,
        duplicates: redundancies.length,
        combinations: combinations.length
      }
    }, null, 2)
  );

  // 3. Pantry JSON (just capabilities)
  fs.writeFileSync(
    path.join(outputDir, 'THE_PANTRY.json'),
    JSON.stringify(pantry, null, 2)
  );

  console.log('✅ Catalog complete!');
  console.log(`   Complete Catalog: ${path.join(outputDir, 'COMPLETE_CATALOG.md')}`);
  console.log(`   JSON Export: ${path.join(outputDir, 'COMPLETE_CATALOG.json')}`);
  console.log(`   The Pantry: ${path.join(outputDir, 'THE_PANTRY.json')}\n`);

  console.log('📊 Summary:');
  console.log(`   Total Repos: ${repos.length}`);
  console.log(`   Total Capabilities: ${pantry.all.length}`);
  console.log(`   Duplicates: ${redundancies.length}`);
  console.log(`   Unique: ${pantry.all.length - redundancies.reduce((sum, d) => sum + d.count - 1, 0)}`);
  console.log(`   Combinations: ${combinations.length}\n`);

  console.log('🍳 Your pantry is ready! Use these ingredients to cook up new products!\n');
}

main().catch(console.error);

