use crate::errors::AlnError;
use crate::session::Session;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

pub async fn run_shell(cmd: &str) -> Result<String, AlnError> {
    let mut parts = shell_words::split(cmd).map_err(|e| AlnError::CommandFailed(e.to_string()))?;
    let binary = parts
        .get(0)
        .cloned()
        .ok_or_else(|| AlnError::CommandFailed("empty command".into()))?;
    let args = &parts[1..];

    let output = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| AlnError::CommandFailed(e.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(AlnError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn session_key_from_template(template: &str, user_id: &str) -> String {
    template.replace("{user_id}", user_id)
}

pub fn json_ok(status: &str, payload: Value) -> Value {
    serde_json::json!({
        "status": status,
        "payload": payload
    })
}

pub fn update_state(session: &mut Session, new_state: &str) {
    session.state = new_state.to_string();
}
