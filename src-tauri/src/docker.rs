use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, PartialEq)]
pub struct Container {
    pub id: String,
    pub image: String,
    pub command: String,
    pub created: String,
    pub status: String,
    pub ports: String,
    pub names: String,
    pub project: String,
}

#[derive(Deserialize)]
struct DockerPsEntry {
    #[serde(rename = "ID", default)]
    id: String,
    #[serde(rename = "Image", default)]
    image: String,
    #[serde(rename = "Command", default)]
    command: String,
    #[serde(rename = "CreatedAt", default)]
    created: String,
    #[serde(rename = "Status", default)]
    status: String,
    #[serde(rename = "Ports", default)]
    ports: String,
    #[serde(rename = "Names", default)]
    names: String,
    #[serde(rename = "Labels", default)]
    labels: String,
}

fn extract_compose_project(labels: &str) -> String {
    labels
        .split(',')
        .find_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            if k.trim() == "com.docker.compose.project" {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

pub fn parse_containers(output: &str) -> Vec<Container> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let entry: DockerPsEntry = serde_json::from_str(line).ok()?;
            Some(Container {
                id: entry.id,
                image: entry.image,
                command: entry.command,
                created: entry.created,
                status: entry.status,
                ports: entry.ports,
                names: entry.names,
                project: extract_compose_project(&entry.labels),
            })
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
    let valid_actions = ["start", "stop", "restart", "down"];
    if !valid_actions.contains(&action) {
        return Err(format!("Invalid action: {}", action));
    }
    Ok(())
}
