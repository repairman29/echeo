use anyhow::Result;
use git2::{Blame, BlameOptions, Repository};
use std::path::Path;

/// Code authorship information extracted from git blame
#[derive(Debug, Clone)]
pub struct AuthorshipInfo {
    pub author_email: Option<String>,
    pub author_name: Option<String>,
    pub commit_sha: Option<String>,
    pub authorship_confidence: f64, // 0.0 to 1.0
    pub is_self_authored: bool,
    pub contribution_percentage: f64, // 0.0 to 100.0
}

/// Analyze code authorship using git blame
pub struct AuthorshipAnalyzer {
    repo: Repository,
    user_email: Option<String>,
    user_name: Option<String>,
}

impl AuthorshipAnalyzer {
    /// Create a new authorship analyzer for a repository
    pub fn new(repo_path: &Path, user_email: Option<String>, user_name: Option<String>) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        Ok(Self {
            repo,
            user_email,
            user_name,
        })
    }

    /// Analyze authorship for a specific file and line range
    pub fn analyze_file(
        &self,
        file_path: &Path,
        start_line: usize,
        end_line: usize,
    ) -> Result<AuthorshipInfo> {
        // Get relative path from repo root
        let repo_path = self.repo.workdir()
            .ok_or_else(|| anyhow::anyhow!("Repository has no workdir"))?;
        
        let relative_path = file_path.strip_prefix(repo_path)
            .or_else(|_| file_path.strip_prefix(repo_path.join(".git")))
            .unwrap_or(file_path);

        // Run git blame
        let mut opts = BlameOptions::new();
        opts.min_line(start_line + 1); // git blame is 1-indexed
        opts.max_line(end_line + 1);
        
        let blame = self.repo.blame_file(relative_path, Some(&mut opts))?;

        // Analyze blame results
        self.analyze_blame(&blame, start_line, end_line)
    }

    /// Analyze authorship for a specific line
    #[allow(dead_code)]
    pub fn analyze_line(&self, file_path: &Path, line: usize) -> Result<AuthorshipInfo> {
        self.analyze_file(file_path, line, line)
    }

    /// Analyze blame results to determine authorship
    fn analyze_blame(
        &self,
        blame: &Blame,
        start_line: usize,
        end_line: usize,
    ) -> Result<AuthorshipInfo> {
        let mut author_emails = Vec::new();
        let mut author_names = Vec::new();
        let mut commit_shas = Vec::new();
        let mut user_lines = 0;
        let total_lines = (end_line - start_line + 1) as f64;

        // Collect authorship info for each line
        for i in start_line..=end_line {
            if let Some(hunk) = blame.get_line(i + 1) {
                let commit = hunk.final_commit_id();
                let commit_obj = self.repo.find_commit(commit)?;
                
                let author = commit_obj.author();
                let email = author.email().map(|e| e.to_string());
                let name = author.name().map(|n| n.to_string());

                if let Some(ref email) = email {
                    author_emails.push(email.clone());
                }
                if let Some(ref name) = name {
                    author_names.push(name.clone());
                }
                commit_shas.push(commit.to_string());

                // Check if this line was written by the user
                let is_user_line = self.is_user_author(&email, &name);
                if is_user_line {
                    user_lines += 1;
                }
            }
        }

        // Determine primary author (most common)
        let primary_email = AuthorshipAnalyzer::most_common(&author_emails);
        let primary_name = AuthorshipAnalyzer::most_common(&author_names);
        let primary_commit = commit_shas.first().cloned();

        // Calculate contribution percentage
        let contribution_percentage = if total_lines > 0.0 {
            (user_lines as f64 / total_lines) * 100.0
        } else {
            100.0
        };

        // Determine if self-authored (>= 80% by user)
        let is_self_authored = contribution_percentage >= 80.0;

        // Calculate confidence based on consistency
        let email_consistency = AuthorshipAnalyzer::calculate_consistency(&author_emails);
        let name_consistency = AuthorshipAnalyzer::calculate_consistency(&author_names);
        let authorship_confidence = (email_consistency + name_consistency) / 2.0;

        Ok(AuthorshipInfo {
            author_email: primary_email,
            author_name: primary_name,
            commit_sha: primary_commit,
            authorship_confidence,
            is_self_authored,
            contribution_percentage,
        })
    }

    /// Check if an author matches the user
    fn is_user_author(&self, email: &Option<String>, name: &Option<String>) -> bool {
        if let Some(ref user_email) = self.user_email {
            if let Some(ref author_email) = email {
                if author_email == user_email || author_email.contains(user_email) {
                    return true;
                }
            }
        }

        if let Some(ref user_name) = self.user_name {
            if let Some(ref author_name) = name {
                if author_name == user_name || author_name.contains(user_name) {
                    return true;
                }
            }
        }

        false
    }

    /// Find most common value in a vector
    fn most_common<T: Clone + Eq + std::hash::Hash>(items: &[T]) -> Option<T> {
        use std::collections::HashMap;
        let mut counts = HashMap::new();
        
        for item in items {
            *counts.entry(item.clone()).or_insert(0) += 1;
        }

        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(item, _)| item)
    }

    /// Calculate consistency score (0.0 to 1.0)
    fn calculate_consistency<T: Eq + std::hash::Hash>(items: &[T]) -> f64 {
        if items.is_empty() {
            return 0.0;
        }

        let total = items.len() as f64;
        let mut counts = std::collections::HashMap::new();
        
        for item in items {
            *counts.entry(item).or_insert(0) += 1;
        }

        let max_count = counts.values().max().copied().unwrap_or(0) as f64;
        max_count / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Note: These tests would require making the helper functions public
    // For now, they're tested indirectly through the analyze_file method
}
