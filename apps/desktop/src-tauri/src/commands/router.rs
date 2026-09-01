use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

// ==================== TASK TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    RequirementAnalysis,
    ArchitectureDesign,
    RepoUnderstanding,
    FrontendCoding,
    BackendCoding,
    DatabaseDesign,
    TestGeneration,
    Debugging,
    CodeReview,
    SecurityReview,
    Documentation,
    Refactoring,
    MultimodalParsing,
    Research,
    Integration,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::RequirementAnalysis => "requirement_analysis",
            TaskType::ArchitectureDesign => "architecture_design",
            TaskType::RepoUnderstanding => "repo_understanding",
            TaskType::FrontendCoding => "frontend_coding",
            TaskType::BackendCoding => "backend_coding",
            TaskType::DatabaseDesign => "database_design",
            TaskType::TestGeneration => "test_generation",
            TaskType::Debugging => "debugging",
            TaskType::CodeReview => "code_review",
            TaskType::SecurityReview => "security_review",
            TaskType::Documentation => "documentation",
            TaskType::Refactoring => "refactoring",
            TaskType::MultimodalParsing => "multimodal_parsing",
            TaskType::Research => "research",
            TaskType::Integration => "integration",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "requirement_analysis" => Some(TaskType::RequirementAnalysis),
            "architecture_design" => Some(TaskType::ArchitectureDesign),
            "repo_understanding" => Some(TaskType::RepoUnderstanding),
            "frontend_coding" => Some(TaskType::FrontendCoding),
            "backend_coding" => Some(TaskType::BackendCoding),
            "database_design" => Some(TaskType::DatabaseDesign),
            "test_generation" => Some(TaskType::TestGeneration),
            "debugging" => Some(TaskType::Debugging),
            "code_review" => Some(TaskType::CodeReview),
            "security_review" => Some(TaskType::SecurityReview),
            "documentation" => Some(TaskType::Documentation),
            "refactoring" => Some(TaskType::Refactoring),
            "multimodal_parsing" => Some(TaskType::MultimodalParsing),
            "research" => Some(TaskType::Research),
            "integration" => Some(TaskType::Integration),
            _ => None,
        }
    }

    /// All variants, in canonical order. Used as single source of truth.
    pub const ALL: &'static [TaskType] = &[
        TaskType::RequirementAnalysis,
        TaskType::ArchitectureDesign,
        TaskType::RepoUnderstanding,
        TaskType::FrontendCoding,
        TaskType::BackendCoding,
        TaskType::DatabaseDesign,
        TaskType::TestGeneration,
        TaskType::Debugging,
        TaskType::CodeReview,
        TaskType::SecurityReview,
        TaskType::Documentation,
        TaskType::Refactoring,
        TaskType::MultimodalParsing,
        TaskType::Research,
        TaskType::Integration,
    ];
}

// ==================== ROUTING INPUT/OUTPUT ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingInput {
    pub task_type: String,
    pub task_complexity: String, // "low", "medium", "high"
    pub required_capabilities: Vec<String>,
    pub max_cost: Option<f64>,
    pub preferred_speed: String, // "fast", "balanced", "quality"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub provider_id: String,
    pub model_profile_id: String,
    pub model_name: String,
    pub score: f64,
    pub reasoning: String,
    pub fallback_provider_id: Option<String>,
}

// ==================== TASK WEIGHTS ====================

// Task type weights for capability scoring
// Format: task_type -> (capability -> weight)
fn get_task_weights(task_type: &str) -> Vec<(&'static str, f64)> {
    match task_type {
        "requirement_analysis" => vec![("reasoning", 5.0), ("coding", 1.0), ("chinese", 3.0)],
        "architecture_design" => vec![("reasoning", 5.0), ("coding", 2.0), ("long_context", 3.0)],
        "repo_understanding" => vec![("reasoning", 4.0), ("long_context", 5.0), ("coding", 2.0)],
        "frontend_coding" => vec![("coding", 5.0), ("tool_use", 3.0), ("speed", 3.0)],
        "backend_coding" => vec![("coding", 5.0), ("reasoning", 3.0), ("speed", 2.0)],
        "database_design" => vec![("coding", 4.0), ("reasoning", 3.0), ("long_context", 2.0)],
        "test_generation" => vec![
            ("coding", 4.0),
            ("tool_use", 4.0),
            ("json_reliability", 3.0),
        ],
        "debugging" => vec![
            ("reasoning", 5.0),
            ("coding", 4.0),
            ("json_reliability", 2.0),
        ],
        "code_review" => vec![("code_review", 5.0), ("reasoning", 3.0), ("coding", 2.0)],
        "security_review" => vec![("code_review", 4.0), ("reasoning", 4.0), ("coding", 2.0)],
        "documentation" => vec![("coding", 2.0), ("chinese", 4.0), ("speed", 3.0)],
        "refactoring" => vec![("coding", 5.0), ("reasoning", 3.0)],
        "multimodal_parsing" => vec![
            ("multimodal", 5.0),
            ("reasoning", 3.0),
            ("long_context", 2.0),
        ],
        "research" => vec![("reasoning", 5.0), ("chinese", 2.0), ("long_context", 3.0)],
        "integration" => vec![("coding", 4.0), ("tool_use", 4.0), ("reasoning", 2.0)],
        _ => vec![("coding", 3.0), ("reasoning", 3.0)],
    }
}

// ==================== MODEL CAPABILITY ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelCapabilityInput {
    pub reasoning: i32,
    pub coding: i32,
    pub code_review: i32,
    pub long_context: i32,
    pub speed: i32,
    pub low_cost: i32,
    pub tool_use: i32,
    pub json_reliability: i32,
    pub multimodal: i32,
    pub chinese: i32,
    pub local_deploy: i32,
    pub rag: i32,
}

impl Default for ModelCapabilityInput {
    fn default() -> Self {
        ModelCapabilityInput {
            reasoning: 3,
            coding: 3,
            code_review: 3,
            long_context: 3,
            speed: 3,
            low_cost: 3,
            tool_use: 3,
            json_reliability: 3,
            multimodal: 0,
            chinese: 3,
            local_deploy: 0,
            rag: 0,
        }
    }
}

// ==================== ROUTING COMMANDS ====================

// Get all available task types
#[tauri::command]
pub fn get_task_types() -> Vec<serde_json::Value> {
    // Single source of truth: iterate over the TaskType enum's ALL slice.
    // This guarantees the UI, router, and agents see exactly the same set.
    let descriptions = [
        (
            "Requirement Analysis",
            "Analyze and break down project requirements",
        ),
        (
            "Architecture Design",
            "Design system architecture and technical decisions",
        ),
        (
            "Repository Understanding",
            "Understand existing codebase structure",
        ),
        (
            "Frontend Coding",
            "Implement UI components and frontend logic",
        ),
        ("Backend Coding", "Implement API and backend services"),
        ("Database Design", "Design database schema and queries"),
        ("Test Generation", "Write unit and integration tests"),
        ("Debugging", "Find and fix bugs"),
        ("Code Review", "Review code quality and best practices"),
        ("Security Review", "Check for security vulnerabilities"),
        ("Documentation", "Write documentation and README"),
        (
            "Refactoring",
            "Improve code structure without changing behavior",
        ),
        (
            "Multimodal Parsing",
            "Parse and understand images, audio, and video inputs",
        ),
        (
            "Research",
            "Investigate technologies, libraries, and best practices",
        ),
        ("Integration", "Integrate different components and services"),
    ];

    TaskType::ALL
        .iter()
        .zip(descriptions.iter())
        .map(|(t, (name, desc))| {
            serde_json::json!({
                "id": t.as_str(),
                "name": name,
                "description": desc,
            })
        })
        .collect()
}

// Route a task to the best model
#[tauri::command]
pub fn route_task(
    db: State<Database>,
    input: RoutingInput,
) -> Result<Vec<RoutingDecision>, String> {
    let conn = db.connection();

    // Get all enabled providers and their models
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.provider_type, m.id as model_id, m.model_id,
                m.context_window, m.input_price, m.output_price,
                c.reasoning, c.coding, c.code_review, c.long_context, c.speed,
                c.low_cost, c.tool_use, c.json_reliability, c.multimodal,
                c.chinese, c.local_deploy, c.rag
         FROM provider_configs p
         LEFT JOIN model_profiles m ON m.provider_id = p.id AND m.is_default = 1
         LEFT JOIN model_capabilities c ON c.model_profile_id = m.id
         WHERE p.is_enabled = 1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<i32>>(5)?,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<f64>>(7)?,
                row.get::<_, Option<i32>>(8)?,
                row.get::<_, Option<i32>>(9)?,
                row.get::<_, Option<i32>>(10)?,
                row.get::<_, Option<i32>>(11)?,
                row.get::<_, Option<i32>>(12)?,
                row.get::<_, Option<i32>>(13)?,
                row.get::<_, Option<i32>>(14)?,
                row.get::<_, Option<i32>>(15)?,
                row.get::<_, Option<i32>>(16)?,
                row.get::<_, Option<i32>>(17)?,
                row.get::<_, Option<i32>>(18)?,
                row.get::<_, Option<i32>>(19)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let weights = get_task_weights(&input.task_type);
    let mut decisions = Vec::new();

    for row_result in rows {
        let row = row_result.map_err(|e| e.to_string())?;

        let (
            provider_id,
            provider_name,
            _provider_type,
            model_id,
            model_id_name,
            context_window,
            input_price,
            output_price,
            reasoning,
            coding,
            code_review,
            long_context,
            speed,
            low_cost,
            tool_use,
            json_reliability,
            multimodal,
            chinese,
            local_deploy,
            rag,
        ) = row;

        // Skip if no model
        let Some(mid) = model_id else { continue };
        let Some(mid_name) = model_id_name else {
            continue;
        };

        // Calculate score based on task weights
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        for (capability, weight) in &weights {
            let value = match *capability {
                "reasoning" => reasoning.unwrap_or(3),
                "coding" => coding.unwrap_or(3),
                "code_review" => code_review.unwrap_or(3),
                "long_context" => long_context.unwrap_or(3),
                "speed" => speed.unwrap_or(3),
                "low_cost" => low_cost.unwrap_or(3),
                "tool_use" => tool_use.unwrap_or(3),
                "json_reliability" => json_reliability.unwrap_or(3),
                "multimodal" => multimodal.unwrap_or(0),
                "chinese" => chinese.unwrap_or(3),
                "local_deploy" => local_deploy.unwrap_or(0),
                "rag" => rag.unwrap_or(0),
                _ => 3,
            };
            total_score += (value as f64) * weight;
            weight_sum += weight;
        }

        // Normalize score
        let final_score = if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            3.0
        };

        // Cost penalty
        let cost_penalty = if let (Some(inp), Some(out)) = (input_price, output_price) {
            let avg_cost = (inp + out) / 2.0;
            // Penalize expensive models slightly
            (5.0 - avg_cost.min(5.0)) * 0.1
        } else {
            0.0
        };

        let adjusted_score = (final_score - cost_penalty).max(0.0);

        decisions.push(RoutingDecision {
            provider_id,
            model_profile_id: mid,
            model_name: mid_name,
            score: adjusted_score,
            reasoning: format!(
                "Provider: {}, Score: {:.2}, Context: {:?}",
                provider_name, adjusted_score, context_window
            ),
            fallback_provider_id: None,
        });
    }

    // Sort by score descending
    decisions.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Set fallback - save first provider_id before loop
    if decisions.len() > 1 {
        let first_provider_id = decisions[0].provider_id.clone();
        for (i, decision) in decisions.iter_mut().enumerate() {
            if i > 0 {
                decision.fallback_provider_id = Some(first_provider_id.clone());
            }
        }
    }

    Ok(decisions)
}

// Save routing decision to history
#[tauri::command]
pub fn save_routing_history(
    db: State<Database>,
    task_id: String,
    decision: RoutingDecision,
) -> Result<(), String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO routing_history
         (id, task_id, selected_provider_id, selected_model_profile_id, final_score, reason, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            task_id,
            decision.provider_id,
            decision.model_profile_id,
            decision.score,
            decision.reasoning,
            Utc::now().to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// Get routing history for a task
#[tauri::command]
pub fn get_routing_history(
    db: State<Database>,
    task_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT id, task_id, selected_provider_id, selected_model_profile_id,
                final_score, reason, created_at
         FROM routing_history
         WHERE task_id = ?1
         ORDER BY created_at DESC
         LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let history = stmt
        .query_map(params![task_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "task_id": row.get::<_, String>(1)?,
                "provider_id": row.get::<_, String>(2)?,
                "model_profile_id": row.get::<_, String>(3)?,
                "score": row.get::<_, f64>(4)?,
                "reason": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(history)
}

// Get all available providers with their models for routing UI
#[tauri::command]
pub fn get_available_models_for_routing(
    db: State<Database>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.provider_type, p.base_url,
                m.id, m.model_id, m.display_model_name, m.context_window,
                c.reasoning, c.coding, c.code_review, c.speed, c.low_cost, c.chinese
         FROM provider_configs p
         LEFT JOIN model_profiles m ON m.provider_id = p.id
         LEFT JOIN model_capabilities c ON c.model_profile_id = m.id
         WHERE p.is_enabled = 1 AND m.id IS NOT NULL
         ORDER BY p.name, m.display_model_name",
        )
        .map_err(|e| e.to_string())?;

    let models = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "provider_id": row.get::<_, String>(0)?,
                "provider_name": row.get::<_, String>(1)?,
                "provider_type": row.get::<_, String>(2)?,
                "base_url": row.get::<_, String>(3)?,
                "model_profile_id": row.get::<_, String>(4)?,
                "model_id": row.get::<_, String>(5)?,
                "display_name": row.get::<_, Option<String>>(6)?,
                "context_window": row.get::<_, Option<i32>>(7)?,
                "capabilities": {
                    "reasoning": row.get::<_, Option<i32>>(8)?,
                    "coding": row.get::<_, Option<i32>>(9)?,
                    "code_review": row.get::<_, Option<i32>>(10)?,
                    "speed": row.get::<_, Option<i32>>(11)?,
                    "low_cost": row.get::<_, Option<i32>>(12)?,
                    "chinese": row.get::<_, Option<i32>>(13)?,
                }
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(models)
}
