use crate::{regex_branch::RegexBranch, codex_branch::CodexBranch, system_branch::SystemBranch, language_branch::LanguageBranch};
use serde_json::Value;

pub struct AlnIntegration;

impl AlnIntegration {
    pub fn integrate_all(user_id: &str) -> Value {
        let regex_res = RegexBranch::integrate_regex("^ALIEN_.*$", "commands")
            .unwrap_or_else(|e| serde_json::json!({ "status": "error", "error": e.to_string() }));
        let codex_res = CodexBranch::generate_code(
            "Generate ALN script for session management",
            "ALN",
        );
        let system_res = SystemBranch::setup_environment("linux", "virtual");
        let language_res = LanguageBranch::define_syntax(
            "@ACTION {.*}",
            "Execute action block",
        )
        .unwrap_or_else(|e| serde_json::json!({ "status": "error", "error": e.to_string() }));

        serde_json::json!({
            "status": "integrated",
            "user_id": user_id,
            "branches": {
                "regex": regex_res,
                "codex": codex_res,
                "system": system_res,
                "language": language_res
            }
        })
    }
}
