use std::process::Command;
use tauri::async_runtime;

use crate::docker::{
    parse_containers, validate_compose_action, validate_container_action, Container,
};

#[tauri::command]
pub async fn get_containers() -> Result<Vec<Container>, String> {
    async_runtime::spawn_blocking(|| {
        let output = Command::new("wsl")
            .args(["bash", "-c", "docker ps -a --format '{{.ID}};;{{.Image}};;{{.Command}};;{{.CreatedAt}};;{{.Status}};;{{.Ports}};;{{.Names}};;{{.Label \"com.docker.compose.project\"}}'"])
            .output()
            .map_err(|e| format!("Failed to execute wsl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Docker command failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_containers(&stdout))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn container_action(id: String, action: String) -> Result<String, String> {
    validate_container_action(&action)?;

    async_runtime::spawn_blocking(move || {
        let output = Command::new("wsl")
            .args(["docker", &action, &id])
            .output()
            .map_err(|e| format!("Failed to execute wsl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Docker {} failed: {}", action, stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn compose_action(project: String, action: String) -> Result<String, String> {
    validate_compose_action(&action)?;

    async_runtime::spawn_blocking(move || {
        let cmd = if action == "start" {
            format!("docker compose -p '{}' start", project)
        } else if action == "stop" {
            format!("docker compose -p '{}' stop", project)
        } else if action == "down" {
            format!("docker compose -p '{}' down", project)
        } else {
            format!("docker compose -p '{}' restart", project)
        };

        let output = Command::new("wsl")
            .args(["bash", "-c", &cmd])
            .output()
            .map_err(|e| format!("Failed to execute wsl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Compose {} failed: {}", action, stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn get_container_logs(id: String, tail: Option<u32>) -> Result<String, String> {
    let tail_count = tail.unwrap_or(200);

    async_runtime::spawn_blocking(move || {
        let output = Command::new("wsl")
            .args(["docker", "logs", "--tail", &tail_count.to_string(), &id])
            .output()
            .map_err(|e| format!("Failed to execute wsl: {}", e))?;

        let mut logs = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            if !logs.is_empty() {
                logs.push('\n');
            }
            logs.push_str(&stderr);
        }

        Ok(logs)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
