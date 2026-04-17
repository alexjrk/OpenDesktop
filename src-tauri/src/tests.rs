#[cfg(test)]
mod tests {
    use crate::docker::*;

    // --- Parsing tests ---

    #[test]
    fn parse_single_container() {
        let output = r#"{"ID":"abc123","Image":"nginx:latest","Command":"nginx -g 'daemon off;'","CreatedAt":"2026-04-10 12:00:00","Status":"Up 2 hours","Ports":"0.0.0.0:80->80/tcp","Names":"my-nginx","Labels":"com.docker.compose.project=myproject"}"#;
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
        let output = concat!(
            r#"{"ID":"aaa","Image":"img1","Command":"cmd1","CreatedAt":"created1","Status":"Up 1 hour","Ports":"80/tcp","Names":"c1","Labels":"com.docker.compose.project=proj"}"#,
            "\n",
            r#"{"ID":"bbb","Image":"img2","Command":"cmd2","CreatedAt":"created2","Status":"Exited (0)","Ports":"","Names":"c2","Labels":"com.docker.compose.project=proj"}"#,
            "\n",
        );
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
        let output = concat!(
            "\n\n",
            r#"{"ID":"aaa","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"ports","Names":"name","Labels":"com.docker.compose.project=proj"}"#,
            "\n\n",
        );
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
    }

    #[test]
    fn parse_missing_fields_default_to_empty() {
        let output = r#"{"ID":"abc123","Image":"nginx"}"#;
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "abc123");
        assert_eq!(containers[0].image, "nginx");
        assert_eq!(containers[0].command, "");
        assert_eq!(containers[0].project, "");
    }

    #[test]
    fn parse_no_compose_project() {
        let output = r#"{"ID":"abc","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"80/tcp","Names":"name","Labels":""}"#;
        let containers = parse_containers(output);
        assert_eq!(containers[0].project, "");
    }

    #[test]
    fn parse_container_with_special_chars_in_name() {
        let output = r#"{"ID":"abc","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"80/tcp","Names":"my-app_web_1","Labels":""}"#;
        let containers = parse_containers(output);
        assert_eq!(containers[0].names, "my-app_web_1");
    }

    #[test]
    fn parse_container_with_multiple_ports() {
        let output = r#"{"ID":"abc","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp","Names":"name","Labels":"com.docker.compose.project=proj"}"#;
        let containers = parse_containers(output);
        assert_eq!(
            containers[0].ports,
            "0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp"
        );
    }

    #[test]
    fn parse_container_empty_ports() {
        let output = r#"{"ID":"abc","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Exited (0)","Ports":"","Names":"name","Labels":"com.docker.compose.project=proj"}"#;
        let containers = parse_containers(output);
        assert_eq!(containers[0].ports, "");
    }

    #[test]
    fn parse_containers_mixed_projects() {
        let output = concat!(
            r#"{"ID":"a","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"80","Names":"c1","Labels":"com.docker.compose.project=proj1"}"#,
            "\n",
            r#"{"ID":"b","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"81","Names":"c2","Labels":"com.docker.compose.project=proj1"}"#,
            "\n",
            r#"{"ID":"c","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Up","Ports":"82","Names":"c3","Labels":""}"#,
            "\n",
        );
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 3);
        assert_eq!(containers[0].project, "proj1");
        assert_eq!(containers[1].project, "proj1");
        assert_eq!(containers[2].project, "");
    }

    #[test]
    fn parse_container_exited_status_with_code() {
        let output = r#"{"ID":"abc","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Exited (137)","Ports":"","Names":"name","Labels":""}"#;
        let containers = parse_containers(output);
        assert_eq!(containers[0].status, "Exited (137)");
    }

    #[test]
    fn parse_container_restarting_status() {
        let output = r#"{"ID":"abc","Image":"img","Command":"cmd","CreatedAt":"created","Status":"Restarting (1) 5 seconds ago","Ports":"","Names":"name","Labels":""}"#;
        let containers = parse_containers(output);
        assert!(containers[0].status.contains("Restarting"));
    }

    #[test]
    fn parse_only_whitespace_output() {
        let containers = parse_containers("   \n  \n   ");
        assert_eq!(containers.len(), 0);
    }

    #[test]
    fn parse_malformed_line_skipped() {
        let output = concat!(
            "not valid json\n",
            r#"{"ID":"abc123","Image":"nginx","Command":"","CreatedAt":"","Status":"Up","Ports":"","Names":"web","Labels":""}"#,
            "\n",
        );
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "abc123");
    }

    // --- Validation tests ---

    #[test]
    fn validate_container_action_valid() {
        assert!(validate_container_action("start").is_ok());
        assert!(validate_container_action("stop").is_ok());
        assert!(validate_container_action("restart").is_ok());
        assert!(validate_container_action("rm").is_ok());
    }

    #[test]
    fn validate_container_action_all_valid() {
        for action in &["start", "stop", "restart", "rm"] {
            assert!(
                validate_container_action(action).is_ok(),
                "Expected {} to be valid",
                action
            );
        }
    }

    #[test]
    fn validate_container_action_invalid() {
        assert!(validate_container_action("delete").is_err());
        assert!(validate_container_action("exec").is_err());
        assert!(validate_container_action("").is_err());
    }

    #[test]
    fn validate_container_action_logs_is_invalid() {
        assert!(validate_container_action("logs").is_err());
    }

    #[test]
    fn validate_compose_action_valid() {
        assert!(validate_compose_action("start").is_ok());
        assert!(validate_compose_action("stop").is_ok());
        assert!(validate_compose_action("restart").is_ok());
    }

    #[test]
    fn validate_compose_action_all_valid() {
        for action in &["start", "stop", "restart", "down"] {
            assert!(
                validate_compose_action(action).is_ok(),
                "Expected {} to be valid",
                action
            );
        }
    }

    #[test]
    fn validate_compose_action_down() {
        assert!(validate_compose_action("down").is_ok());
    }

    #[test]
    fn validate_compose_action_invalid() {
        assert!(validate_compose_action("rm").is_err());
        assert!(validate_compose_action("").is_err());
    }

    #[test]
    fn validate_compose_action_up_is_invalid() {
        assert!(validate_compose_action("up").is_err());
    }
}
