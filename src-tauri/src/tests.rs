#[cfg(test)]
mod tests {
    use crate::docker::*;

    // --- Parsing tests ---

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
    fn parse_container_with_special_chars_in_name() {
        let output = "abc;;img;;cmd;;created;;Up;;80/tcp;;my-app_web_1;;\n";
        let containers = parse_containers(output);
        assert_eq!(containers[0].names, "my-app_web_1");
    }

    #[test]
    fn parse_container_with_multiple_ports() {
        let output =
            "abc;;img;;cmd;;created;;Up;;0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp;;name;;proj\n";
        let containers = parse_containers(output);
        assert_eq!(
            containers[0].ports,
            "0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp"
        );
    }

    #[test]
    fn parse_container_empty_ports() {
        let output = "abc;;img;;cmd;;created;;Exited (0);;;;name;;proj\n";
        let containers = parse_containers(output);
        assert_eq!(containers[0].ports, "");
    }

    #[test]
    fn parse_containers_mixed_projects() {
        let output = "a;;img;;cmd;;created;;Up;;80;;c1;;proj1\nb;;img;;cmd;;created;;Up;;81;;c2;;proj1\nc;;img;;cmd;;created;;Up;;82;;c3;;\n";
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 3);
        assert_eq!(containers[0].project, "proj1");
        assert_eq!(containers[1].project, "proj1");
        assert_eq!(containers[2].project, "");
    }

    #[test]
    fn parse_container_exited_status_with_code() {
        let output = "abc;;img;;cmd;;created;;Exited (137);;;;name;;\n";
        let containers = parse_containers(output);
        assert_eq!(containers[0].status, "Exited (137)");
    }

    #[test]
    fn parse_container_restarting_status() {
        let output = "abc;;img;;cmd;;created;;Restarting (1) 5 seconds ago;;;;name;;\n";
        let containers = parse_containers(output);
        assert!(containers[0].status.contains("Restarting"));
    }

    #[test]
    fn parse_only_whitespace_output() {
        let containers = parse_containers("   \n  \n   ");
        assert_eq!(containers.len(), 0);
    }

    #[test]
    fn parse_single_field_line() {
        let output = "abc123\n";
        let containers = parse_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "abc123");
        assert_eq!(containers[0].image, "");
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
