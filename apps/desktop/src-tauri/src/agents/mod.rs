use crate::context::TaskContext;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PromptBuilder {
    agent: AgentConfig,
    task_type: String,
    task_title: String,
    task_description: Option<String>,
    project_path: String,
    context: Option<TaskContext>,
}

impl PromptBuilder {
    pub fn new(
        agent: AgentConfig,
        task_type: String,
        task_title: String,
        task_description: Option<String>,
        project_path: String,
    ) -> Self {
        Self {
            agent,
            task_type,
            task_title,
            task_description,
            project_path,
            context: None,
        }
    }

    pub fn with_context(mut self, context: TaskContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn build(&self) -> String {
        let mut prompt = String::new();

        // System prompt
        prompt.push_str(&self.agent.system_prompt);
        prompt.push_str("\n\n");

        // Task instruction
        prompt.push_str(&format!(
            "## Task\n\nTitle: {}\nType: {}\n",
            self.task_title, self.task_type
        ));

        if let Some(desc) = &self.task_description {
            prompt.push_str(&format!("Description: {}\n", desc));
        }

        prompt.push_str(&format!("Project Path: {}\n", self.project_path));

        // Context files
        if let Some(context) = &self.context {
            if !context.files.is_empty() {
                prompt.push_str("\n## Relevant Files\n\n");
                let mut sorted_files = context.files.clone();
                sorted_files.sort_by(|a, b| {
                    b.relevance
                        .partial_cmp(&a.relevance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for file in sorted_files.iter().take(10) {
                    prompt.push_str(&format!(
                        "### {}\n```\n{}\n```\n\n",
                        file.path,
                        if file.content.len() > 5000 {
                            format!("{}...\n[truncated]", &file.content[..5000])
                        } else {
                            file.content.clone()
                        }
                    ));
                }
            }
        }

        // Task-specific instructions
        prompt.push_str(&self.task_specific_instructions());
        prompt.push_str("\n\n## Output Format\n\nPlease provide your response. For code changes, output them as unified diff patches that can be applied to the files.");

        prompt
    }

    fn task_specific_instructions(&self) -> String {
        match self.task_type.as_str() {
            "requirement_analysis" => {
                "\n## Instructions\n\nAnalyze the requirements and provide:\n1. Clear understanding of the goal\n2. Key requirements list\n3. Acceptance criteria\n4. Potential edge cases".to_string()
            }
            "architecture_design" => {
                "\n## Instructions\n\nDesign the system architecture:\n1. High-level architecture overview\n2. Key components and their responsibilities\n3. Data flow\n4. Technology choices and rationale".to_string()
            }
            "frontend_coding" => {
                "\n## Instructions\n\nImplement the frontend code:\n1. Component structure\n2. State management approach\n3. UI/UX considerations\n4. Output code as unified diff patches".to_string()
            }
            "backend_coding" => {
                "\n## Instructions\n\nImplement the backend code:\n1. API design\n2. Business logic\n3. Error handling\n4. Output code as unified diff patches".to_string()
            }
            "test_generation" => {
                "\n## Instructions\n\nWrite tests:\n1. Unit tests for key functions\n2. Integration tests for API endpoints\n3. Edge case coverage\n4. Output test code as unified diff patches".to_string()
            }
            "debugging" => {
                "\n## Instructions\n\nAnalyze and fix the bug:\n1. Root cause analysis\n2. Fix implementation\n3. Verification approach\n4. Output fix as unified diff patches".to_string()
            }
            "code_review" => {
                "\n## Instructions\n\nReview the code:\n1. Code quality assessment\n2. Potential issues\n3. Improvement suggestions\n4. Best practices adherence".to_string()
            }
            "documentation" => {
                "\n## Instructions\n\nWrite documentation:\n1. Overview\n2. Usage examples\n3. API documentation\n4. Implementation notes".to_string()
            }
            _ => {
                "\n## Instructions\n\nComplete the task following best practices. Output any code changes as unified diff patches.".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptOutput {
    pub system_prompt: String,
    pub user_prompt: String,
}

impl PromptBuilder {
    pub fn build_split(&self) -> AgentPromptOutput {
        AgentPromptOutput {
            system_prompt: self.agent.system_prompt.clone(),
            user_prompt: self.build(),
        }
    }
}

pub fn extract_keywords(task_title: &str, task_description: Option<&str>) -> Vec<String> {
    let mut text = task_title.to_string();
    if let Some(desc) = task_description {
        text.push(' ');
        text.push_str(desc);
    }

    // Simple keyword extraction: split by non-alphanumeric, filter short words
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 3)
        .map(|s| s.to_lowercase())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}
