use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Debug, PartialEq)]
pub struct Container {
    id: String,
    image: String,
    command: String,
    created: String,
    status: String,
    ports: String,
    names: String,
    project: String,
}

pub fn parse_containers(output: &str) -> Vec<Container> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split(";;").collect();
            Container {
                id: parts.first().unwrap_or(&"").to_string(),
                image: parts.get(1).unwrap_or(&"").to_string(),
                command: parts.get(2).unwrap_or(&"").to_string(),
                created: parts.get(3).unwrap_or(&"").to_string(),
                status: parts.get(4).unwrap_or(&"").to_string(),
                ports: parts.get(5).unwrap_or(&"").to_string(),
                names: parts.get(6).unwrap_or(&"").to_string(),
                project: parts.get(7).unwrap_or(&"").to_string(),
            }
        })
        .collect()
}

pub fn validate_container_action(action: &str) -> Result<(), String> {
    let valid_actions = ["start", "stop", "restart", "rm"];
    if !valid_actions.contains(&action) {
        return Err(format!("Invalid action: {}", action));
    }
    Ok(())
}

pub fn validate_compose_action(action: &str) -> Result<(), String> {
    let valid_actions = ["start", "stop", "restart"];
    if !valid_actions.contains(&action) {
        return Err(format!("Invalid action: {}", action));
    }
    Ok(())
}

#[tauri::command]
fn get_containers() -> Result<Vec<Container>, String> {
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
}

#[tauri::command]
fn container_action(id: String, action: String) -> Result<String, String> {
    validate_container_action(&action)?;

    let output = Command::new("wsl")
        .args(["docker", &action, &id])
        .output()
        .map_err(|e| format!("Failed to execute wsl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Docker {} failed: {}", action, stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[tauri::command]
fn compose_action(project: String, action: String) -> Result<String, String> {
    validate_compose_action(&action)?;

    let cmd = if action == "start" {
        format!("docker compose -p '{}' start", project)
    } else if action == "stop" {
        format!("docker compose -p '{}' stop", project)
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
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_containers,
            container_action,
            compose_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_container() {
        let output = "abc123;;nginx:latest;;\"nginx -g 'daemon off;'\";;2026-04-10 12:00:00;;Up 2 hours;;0.0.0.0:80->80/tcp;;my-nginx;;myproject\n";
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "abc123");
        assert_eq!(containers[0].image, "nginx:latest");
        assert_eq!(containers[0].names, "my-nginx");
        assert_eq!(containers[0].project, "myproject");
        assert_eq!(containers[0].ports, "0.0.0.0:80->80/tcp");
    }

    #[test]
    fn parse_multiple_containers() {
        let output = "aaa;;img1;;cmd1;;created1;;Up 1 hour;;80/tcp;;c1;;proj\nbbb;;img2;;cmd2;;created2;;Exited (0);;;;c2;;proj\n";
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].id, "aaa");
        assert_eq!(containers[1].id, "bbb");
        assert_eq!(containers[1].status, "Exited (0)");
    }

    #[test]
    fn parse_empty_output() {
        let containers = parse_containers("");
        assert_eq!(containers.len(), 0);
    }

    #[test]
    fn parse_blank_lines_ignored() {
        let output = "\n\naaa;;img;;cmd;;created;;Up;;ports;;name;;proj\n\n";
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
    }

    #[test]
    fn parse_missing_fields_default_to_empty() {
        let output = "abc123;;nginx\n";
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "abc123");
        assert_eq!(containers[0].image, "nginx");
        assert_eq!(containers[0].command, "");
        assert_eq!(containers[0].project, "");
    }

    #[test]
    fn parse_no_compose_project() {
        let output = "abc;;img;;cmd;;created;;Up;;80/tcp;;name;;\n";
        let containers = parse_containers(output);
        assert_eq!(containers[0].project, "");
    }

    #[test]
    fn validate_container_action_valid() {
        assert!(validate_container_action("start").is_ok());
        assert!(validate_container_action("stop").is_ok());
        assert!(validate_container_action("restart").is_ok());
        assert!(validate_container_action("rm").is_ok());
    }

    #[test]
    fn validate_container_action_invalid() {
        assert!(validate_container_action("delete").is_err());
        assert!(validate_container_action("exec").is_err());
        assert!(validate_container_action("").is_err());
    }

    #[test]
    fn validate_compose_action_valid() {
        assert!(validate_compose_action("start").is_ok());
        assert!(validate_compose_action("stop").is_ok());
        assert!(validate_compose_action("restart").is_ok());
    }

    #[test]
    fn validate_compose_action_invalid() {
        assert!(validate_compose_action("rm").is_err());
        assert!(validate_compose_action("down").is_err());
        assert!(validate_compose_action("").is_err());
    }
}
