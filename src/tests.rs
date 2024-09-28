#[cfg(test)]
mod tests {
    use crate::{is_shell_builtin, should_execute_command};

    #[test]
    fn test_is_shell_builtin() {
        assert!(is_shell_builtin("cd"));
        assert!(is_shell_builtin("cd /home"));
        assert!(is_shell_builtin("export PATH=/usr/bin"));
        assert!(is_shell_builtin("alias ll='ls -la'"));
        assert!(!is_shell_builtin("ls -la"));
        assert!(!is_shell_builtin("grep 'test' file.txt"));
        assert!(!is_shell_builtin(""));
        assert!(!is_shell_builtin(" "));
    }

    #[test]
    fn test_should_execute_command() {
        // Commands that should not be executed
        assert!(should_execute_command("cd /home").is_err());
        assert!(should_execute_command("export VAR=value").is_err());
        assert!(should_execute_command("alias ll='ls -la'").is_err());

        // Commands that should be executed
        assert!(should_execute_command("ls -la").is_ok());
        assert!(should_execute_command("grep 'test' file.txt").is_ok());
    }
}