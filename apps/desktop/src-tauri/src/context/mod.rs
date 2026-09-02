use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: String,
    pub project_path: String,
    pub files: Vec<FileContext>,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct ContextBuilder {
    project_path: String,
    max_file_size: u64,
    max_total_size: u64,
    sensitive_patterns: Vec<String>,
}

impl ContextBuilder {
    pub fn new(project_path: String) -> Self {
        Self {
            project_path,
            max_file_size: 100 * 1024,  // 100KB per file
            max_total_size: 500 * 1024, // 500KB total
            sensitive_patterns: vec![
                ".env".to_string(),
                ".env.*".to_string(),
                "*.pem".to_string(),
                "*.key".to_string(),
                "id_rsa".to_string(),
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
                "target".to_string(),
            ],
        }
    }

    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn with_max_total_size(mut self, size: u64) -> Self {
        self.max_total_size = size;
        self
    }

    pub fn build(&self, task_id: &str, keywords: &[String]) -> Result<TaskContext, String> {
        let project_path = Path::new(&self.project_path);
        let canonical_project_path = project_path
            .canonicalize()
            .map_err(|_| "Project path does not exist".to_string())?;
        let mut files = Vec::new();
        let mut total_size = 0u64;

        self.collect_files_recursive(
            &canonical_project_path,
            &canonical_project_path,
            keywords,
            &mut files,
            &mut total_size,
            0,
        )?;

        Ok(TaskContext {
            task_id: task_id.to_string(),
            project_path: self.project_path.clone(),
            files,
            total_size,
        })
    }

    fn collect_files_recursive(
        &self,
        dir: &Path,
        project_root: &Path,
        keywords: &[String],
        files: &mut Vec<FileContext>,
        total_size: &mut u64,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 5 || *total_size >= self.max_total_size {
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if crate::security::is_sensitive_path(&path) || self.is_sensitive(&name) {
                continue;
            }

            // Context is submitted to a provider, so never follow symlinks.
            // Besides preventing project-root escapes, this avoids recursive
            // links and surprising inclusion of files outside the workspace.
            let link_metadata = match entry.file_type() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if link_metadata.is_symlink() {
                continue;
            }

            let canonical_path = match path.canonicalize() {
                Ok(path) if path.starts_with(project_root) => path,
                _ => continue,
            };
            let metadata = match fs::metadata(&canonical_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                self.collect_files_recursive(
                    &canonical_path,
                    project_root,
                    keywords,
                    files,
                    total_size,
                    depth + 1,
                )?;
            } else {
                let size = metadata.len();
                if size > self.max_file_size {
                    continue;
                }

                if *total_size + size > self.max_total_size {
                    continue;
                }

                let content = match fs::read_to_string(&canonical_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let relevance = self.calculate_relevance(&name, &content, keywords);

                files.push(FileContext {
                    path: canonical_path.to_string_lossy().to_string(),
                    content,
                    size,
                    relevance,
                });

                *total_size += size;
            }
        }

        Ok(())
    }

    fn is_sensitive(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        if name_lower == ".env" || name_lower.starts_with(".env.") {
            return true;
        }
        for pattern in &self.sensitive_patterns {
            if pattern.starts_with("*.") {
                let ext = &pattern[1..];
                if name_lower.ends_with(ext) {
                    return true;
                }
            } else if name_lower == pattern.to_lowercase()
                || name_lower.contains(&pattern.to_lowercase())
            {
                return true;
            }
        }
        matches!(name_lower.as_str(), "id_ed25519" | "credentials.json")
            || (name_lower.starts_with("service-account") && name_lower.ends_with(".json"))
            || [".p12", ".pfx"].iter().any(|ext| name_lower.ends_with(ext))
    }

    fn calculate_relevance(&self, name: &str, content: &str, keywords: &[String]) -> f32 {
        if keywords.is_empty() {
            return 0.5;
        }

        let mut score = 0.0;
        let content_lower = content.to_lowercase();
        let name_lower = name.to_lowercase();

        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();

            // Filename match: high relevance
            if name_lower.contains(&keyword_lower) {
                score += 2.0;
            }

            // Content match
            let occurrences = content_lower.matches(&keyword_lower).count();
            score += (occurrences as f32).min(5.0) * 0.1;
        }

        score
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn excludes_symlinked_files_from_context() {
        let temp =
            std::env::temp_dir().join(format!("supercompany-context-{}", uuid::Uuid::new_v4()));
        let project = temp.join("project");
        let outside = temp.join("outside.txt");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("inside.txt"), "inside").unwrap();
        fs::write(&outside, "sensitive outside content").unwrap();
        symlink(&outside, project.join("outside-link.txt")).unwrap();

        let context = ContextBuilder::new(project.to_string_lossy().to_string())
            .build("task", &[])
            .unwrap();

        assert_eq!(context.files.len(), 1);
        assert_eq!(context.files[0].content, "inside");
        fs::remove_dir_all(&temp).unwrap();
    }
}
