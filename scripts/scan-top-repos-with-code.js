/**
 * Scan Top Repos with Code Snippets
 * Re-scans top repositories without --skip-embeddings to get loadout.json with code
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const { Octokit } = require('@octokit/rest');

async function getGitHubToken() {
  // Try to load from Supabase or env
  try {
    const { createClient } = require('@supabase/supabase-js');
    const supabaseUrl = process.env.SUPABASE_URL || process.env.NEXT_PUBLIC_SUPABASE_URL;
    const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY;

    if (supabaseUrl && supabaseKey) {
      const supabase = createClient(supabaseUrl, supabaseKey);
      const { data } = await supabase
        .from('app_config')
        .select('value')
        .eq('key', 'github_token')
        .single();

      if (data && data.value) {
        return data.value;
      }
    }
  } catch (error) {
    // Fall back to env
  }

  return process.env.GITHUB_TOKEN || null;
}

async function getAllRepos(octokit) {
  const repos = [];
  let page = 1;
  let hasMore = true;

  while (hasMore) {
    try {
      const { data } = await octokit.repos.listForAuthenticatedUser({
        per_page: 100,
        page: page,
        affiliation: 'owner,collaborator,organization_member',
        sort: 'updated',
        direction: 'desc'
      });

      if (data.length === 0) {
        hasMore = false;
      } else {
        repos.push(...data);
        page++;
      }
    } catch (error) {
      console.error('Error fetching repos:', error.message);
      hasMore = false;
    }
  }

  return repos;
}

function scanRepoWithCode(repo, githubToken, tempDir) {
  return new Promise((resolve) => {
    const repoName = repo.name;
    const cloneDir = path.join(tempDir, repoName);
    
    // Clean up if exists
    if (fs.existsSync(cloneDir)) {
      try {
        execSync(`rm -rf "${cloneDir}"`, { stdio: 'ignore' });
      } catch (e) {
        // Ignore
      }
    }

    try {
      // Clone the repo
      const cloneUrl = repo.clone_url.replace('https://', `https://${githubToken}@`);
      execSync(`git clone --depth 1 "${cloneUrl}" "${cloneDir}"`, { 
        stdio: 'ignore',
        timeout: 120000 // 2 minute timeout
      });

      // Scan with echeo WITHOUT --skip-embeddings to get code snippets
      const echeoProcess = spawn('echeo', [
        '--path', cloneDir,
        '--generate-loadout'
        // Note: NOT using --skip-embeddings, so we get full capability data
      ], {
        cwd: cloneDir,
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
        // Check for loadout.json
        const loadoutPath = path.join(cloneDir, '.echeo', 'loadout.json');
        const altLoadoutPath = path.join(cloneDir, 'loadout.json');
        const payloadLoadoutPath = path.join(cloneDir, '.payload', 'loadout.json');
        
        let loadout = null;
        let loadoutPathFound = null;

        for (const lp of [loadoutPath, altLoadoutPath, payloadLoadoutPath]) {
          if (fs.existsSync(lp)) {
            try {
              loadout = JSON.parse(fs.readFileSync(lp, 'utf8'));
              loadoutPathFound = lp;
              break;
            } catch (e) {
              // Try next path
            }
          }
        }

        const capabilities = loadout?.armory || [];
        const capabilityData = capabilities.map(cap => ({
          name: cap.name,
          path: cap.path,
          tags: cap.tags || [],
          confidence: cap.confidence || 0.98
        }));

        // Clean up clone
        try {
          execSync(`rm -rf "${cloneDir}"`, { stdio: 'ignore' });
        } catch (e) {
          // Ignore cleanup errors
        }

        resolve({
          repo: repo.full_name,
          capabilities: capabilities.length,
          capabilityData: capabilityData,
          success: capabilities.length > 0,
          stdout,
          stderr,
          loadoutPath: loadoutPathFound
        });
      });

      echeoProcess.on('error', (error) => {
        // Clean up on error
        try {
          if (fs.existsSync(cloneDir)) {
            execSync(`rm -rf "${cloneDir}"`, { stdio: 'ignore' });
          }
        } catch (e) {
          // Ignore
        }

        resolve({
          repo: repo.full_name,
          capabilities: 0,
          capabilityData: [],
          success: false,
          stdout: '',
          stderr: error.message,
          error: error.message
        });
      });
    } catch (error) {
      // Clean up on error
      try {
        if (fs.existsSync(cloneDir)) {
          execSync(`rm -rf "${cloneDir}"`, { stdio: 'ignore' });
        }
      } catch (e) {
        // Ignore
      }

      resolve({
        repo: repo.full_name,
        capabilities: 0,
        capabilityData: [],
        success: false,
        stdout: '',
        stderr: '',
        error: error.message
      });
    }
  });
}

async function main() {
  console.log('🚀 Scanning Top Repos with Code Snippets\n');
  console.log('='.repeat(60));

  const githubToken = await getGitHubToken();
  if (!githubToken) {
    console.error('❌ No GitHub token found.');
    process.exit(1);
  }

  const octokit = new Octokit({ auth: githubToken });

  // Get all repos
  console.log('📋 Fetching repositories...');
  const repos = await getAllRepos(octokit);
  console.log(`✅ Found ${repos.length} repositories\n`);

  // Sort by size (largest first) or stars, take top 10
  const topRepos = repos
    .filter(r => !r.archived && r.size > 0)
    .sort((a, b) => (b.stargazers_count || 0) - (a.stargazers_count || 0))
    .slice(0, 10);

  console.log(`🎯 Scanning top ${topRepos.length} repos for code snippets...\n`);

  // Check Echeo CLI
  try {
    execSync('which echeo', { stdio: 'ignore' });
  } catch (e) {
    console.error('❌ Echeo CLI not found. Install with: npm install -g echeo');
    process.exit(1);
  }

  // Create temp directory
  const tempDir = path.join(__dirname, '../.temp-scans');
  if (!fs.existsSync(tempDir)) {
    fs.mkdirSync(tempDir, { recursive: true });
  }

  const results = [];
  let totalCapabilities = 0;

  for (let i = 0; i < topRepos.length; i++) {
    const repo = topRepos[i];
    console.log(`[${i + 1}/${topRepos.length}] ${repo.full_name}${repo.private ? ' 🔒' : ''}...`);

    const result = await scanRepoWithCode(repo, githubToken, tempDir);
    results.push({
      ...repo,
      ...result
    });

    totalCapabilities += result.capabilities;

    if (result.success) {
      console.log(`   ✅ ${result.capabilities} capabilities with code snippets`);
    } else {
      console.log(`   ⚠️  ${result.capabilities} capabilities (${result.error || 'scan failed'})`);
    }

    // Rate limiting
    if (i < topRepos.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 3000));
    }
  }

  // Save results
  const outputDir = path.join(__dirname, '../docs/repo-catalog');
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const outputPath = path.join(outputDir, 'TOP_REPOS_WITH_CODE.json');
  fs.writeFileSync(outputPath, JSON.stringify({
    scannedAt: new Date().toISOString(),
    totalRepos: topRepos.length,
    totalCapabilities: totalCapabilities,
    repos: results
  }, null, 2));

  console.log('\n' + '='.repeat(60));
  console.log('✅ Scan complete!');
  console.log(`   Total capabilities: ${totalCapabilities}`);
  console.log(`   Results saved to: ${outputPath}\n`);

  // Clean up temp directory
  try {
    execSync(`rm -rf "${tempDir}"`, { stdio: 'ignore' });
  } catch (e) {
    // Ignore
  }
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = { main };

