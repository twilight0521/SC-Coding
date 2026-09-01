use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPatch {
    pub file_path: String,
    pub patch: String,
    pub change_type: PatchChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatchChangeType {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchParseResult {
    pub patches: Vec<ParsedPatch>,
    pub summary: Option<String>,
    pub has_errors: bool,
    pub error_message: Option<String>,
}

/// Parse a model's output for unified diff patches.
/// Supports the following formats:
/// 1. ```diff ... ``` blocks
/// 2. ```patch ... ``` blocks
/// 3. Direct unified diff format
pub struct PatchParser;

impl PatchParser {
    pub fn parse(output: &str) -> PatchParseResult {
        let mut patches = Vec::new();
        let mut summary = None;
        let mut has_errors = false;
        let mut error_message = None;

        // Try to find diff code blocks
        let diff_patterns = ["```diff", "```patch", "```diff ", "```patch "];

        for pattern in &diff_patterns {
            if let Some(blocks) = Self::extract_code_blocks(output, pattern) {
                for block in blocks {
                    if let Some(patch) = Self::parse_diff_block(&block) {
                        patches.push(patch);
                    }
                }
            }
        }

        // Try to find JSON-formatted patches
        if let Some(json_blocks) = Self::extract_code_blocks(output, "```json") {
            for block in json_blocks {
                if let Ok(parsed) = serde_json::from_str::<Vec<ParsedPatch>>(&block) {
                    patches.extend(parsed);
                }
            }
        }

        // Extract summary (text before code blocks)
        if let Some(idx) = output.find("```") {
            summary = Some(output[..idx].trim().to_string());
        }

        if patches.is_empty() {
            has_errors = true;
            error_message = Some("No patches found in output".to_string());
        }

        PatchParseResult {
            patches,
            summary,
            has_errors,
            error_message,
        }
    }

    fn extract_code_blocks(text: &str, start_marker: &str) -> Option<Vec<String>> {
        let mut blocks = Vec::new();
        let mut start = 0;

        while let Some(pos) = text[start..].find(start_marker) {
            let abs_pos = start + pos;
            let content_start = abs_pos + start_marker.len();

            // Find the end marker
            if let Some(end_pos) = text[content_start..].find("```") {
                let block = text[content_start..content_start + end_pos].trim();
                blocks.push(block.to_string());
                start = content_start + end_pos + 3;
            } else {
                break;
            }
        }

        if blocks.is_empty() {
            None
        } else {
            Some(blocks)
        }
    }

    fn parse_diff_block(block: &str) -> Option<ParsedPatch> {
        let mut file_path = String::new();
        let mut change_type = PatchChangeType::Modify;

        for line in block.lines() {
            // Extract file path from diff header
            if line.starts_with("+++ ") {
                file_path = line[4..].trim().trim_start_matches("b/").to_string();
            } else if line.starts_with("--- ") {
                let p = line[4..].trim().trim_start_matches("a/");
                if p != "/dev/null" {
                    if file_path.is_empty() {
                        file_path = p.to_string();
                    }
                } else {
                    change_type = PatchChangeType::Delete;
                }
            } else if line.starts_with("new file") {
                change_type = PatchChangeType::Create;
            } else if line.starts_with("deleted file") {
                change_type = PatchChangeType::Delete;
            }
        }

        if file_path.is_empty() {
            return None;
        }

        Some(ParsedPatch {
            file_path,
            patch: block.to_string(),
            change_type,
        })
    }

    /// Apply a unified diff patch to a file. Returns the new content.
    ///
    /// Handles context (leading space), removed (`-`), and added (`+`) lines,
    /// preserves the region of the file before the first hunk, and skips
    /// `---`/`+++`/`\ No newline` marker lines.
    pub fn apply_patch(original: &str, patch: &str) -> Result<String, String> {
        let context: Vec<&str> = original.lines().collect();
        let mut output: Vec<&str> = Vec::new();
        let mut src_idx: usize = 0;
        let mut in_hunk = false;

        for line in patch.lines() {
            if line.starts_with("@@") {
                let old_start = parse_hunk_old_start(line);
                // Copy any region of the original file that precedes this hunk.
                while src_idx < old_start.saturating_sub(1) {
                    if src_idx < context.len() {
                        output.push(context[src_idx]);
                        src_idx += 1;
                    } else {
                        break;
                    }
                }
                in_hunk = true;
            } else if line.starts_with('\\') {
                // "\ No newline at end of file" — ignore.
                continue;
            } else if let Some(content) = line.strip_prefix('+') {
                output.push(content);
            } else if let Some(content) = line.strip_prefix('-') {
                // Consume the original line being removed.
                let _ = content;
                src_idx += 1;
            } else if line.starts_with(' ') {
                if src_idx < context.len() {
                    output.push(context[src_idx]);
                    src_idx += 1;
                }
            } else {
                // Any non-hunk, non-marker line (e.g. `---`/`+++`) is skipped.
                let _ = in_hunk;
            }
        }

        // Append the remainder of the original file.
        while src_idx < context.len() {
            output.push(context[src_idx]);
            src_idx += 1;
        }

        Ok(join_lines(output))
    }
}

/// Parse the old-file start line from a `@@ -oldStart,oldCount +newStart,newCount @@` header.
fn parse_hunk_old_start(line: &str) -> usize {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        if let Some(range) = part.strip_prefix('-') {
            let start = range.split(',').next().unwrap_or("0");
            return start.parse().unwrap_or(0);
        }
    }
    0
}

/// Join lines back into a string with trailing newline preserved.
fn join_lines(lines: Vec<&str>) -> String {
    let mut result = lines.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_create_patch() {
        let patch = "@@ -0,0 +1,2 @@\n+hello\n+world\n";
        assert_eq!(
            PatchParser::apply_patch("", patch).unwrap(),
            "hello\nworld\n"
        );
    }

    #[test]
    fn apply_modify_patch_in_middle() {
        let original = "line1\nline2\nline3\n";
        let patch = "@@ -1,3 +1,2 @@\n line1\n-line2\n+changed\n";
        assert_eq!(
            PatchParser::apply_patch(original, patch).unwrap(),
            "line1\nchanged\nline3\n"
        );
    }

    #[test]
    fn apply_leading_region_preserved() {
        // A hunk that only touches lines 3-4 must keep lines 1-2 intact.
        let original = "a\nb\nc\nd\ne\n";
        let patch = "@@ -3,2 +3,2 @@\n c\n-d\n+D\n";
        assert_eq!(
            PatchParser::apply_patch(original, patch).unwrap(),
            "a\nb\nc\nD\ne\n"
        );
    }

    #[test]
    fn apply_delete_patch() {
        let original = "a\nb\nc\n";
        let patch = "@@ -1,3 +0,0 @@\n-a\n-b\n-c\n";
        assert_eq!(PatchParser::apply_patch(original, patch).unwrap(), "");
    }
}
