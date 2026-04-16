use serde::Serialize;

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
    let valid_actions = ["start", "stop", "restart", "down"];
    if !valid_actions.contains(&action) {
        return Err(format!("Invalid action: {}", action));
    }
    Ok(())
}
