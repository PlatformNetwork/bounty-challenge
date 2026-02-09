//! Shell utility module for cross-platform command conversion.
//!
//! Provides functions for converting between bash and PowerShell syntax,
//! detecting the current shell environment, and executing commands in a
//! platform-appropriate manner.
//!
//! # Supported conversions
//!
//! - Environment variables: `$HOME` -> `$env:HOME`
//! - Subshell execution: `$(cmd)` -> `$(cmd)` (PowerShell compatible)
//! - Special variables: `$?` -> `$LASTEXITCODE`, `$$` -> `$PID`
//! - Command substitution, string literals, and more
//!
//! # Examples
//!
//! ```rust
//! use bounty_challenge::shell::powershell;
//!
//! let ps = powershell::from_bash("echo $HOME");
//! assert!(ps.contains("$env:HOME"));
//! ```

/// Detect the current shell type based on environment variables.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Unknown(String),
}

impl ShellType {
    /// Detect the current shell from the SHELL environment variable.
    pub fn detect() -> Self {
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("bash") {
                ShellType::Bash
            } else if shell.contains("zsh") {
                ShellType::Zsh
            } else if shell.contains("fish") {
                ShellType::Fish
            } else {
                ShellType::Unknown(shell)
            }
        } else if std::env::var("PSModulePath").is_ok() {
            ShellType::PowerShell
        } else if std::env::var("COMSPEC").is_ok() {
            ShellType::Cmd
        } else {
            ShellType::Unknown("unknown".to_string())
        }
    }

    /// Returns true if the shell is a POSIX-compatible shell.
    pub fn is_posix(&self) -> bool {
        matches!(self, ShellType::Bash | ShellType::Zsh)
    }

    /// Returns true if the shell is a Windows shell.
    pub fn is_windows(&self) -> bool {
        matches!(self, ShellType::PowerShell | ShellType::Cmd)
    }
}

/// Bash-specific utilities for parsing and manipulating shell commands.
pub mod bash {
    /// Escape a string for safe use in a bash command.
    pub fn escape(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 10);
        result.push('\'');
        for c in s.chars() {
            if c == '\'' {
                result.push_str("'\\''");
            } else {
                result.push(c);
            }
        }
        result.push('\'');
        result
    }

    /// Split a bash command string into tokens, respecting quotes.
    pub fn tokenize(cmd: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        for c in cmd.chars() {
            if escape_next {
                current.push(c);
                escape_next = false;
                continue;
            }

            if c == '\\' && !in_single_quote {
                escape_next = true;
                current.push(c);
                continue;
            }

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                current.push(c);
                continue;
            }

            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                current.push(c);
                continue;
            }

            if c.is_whitespace() && !in_single_quote && !in_double_quote {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                continue;
            }

            current.push(c);
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    /// Check if a string looks like a valid bash variable name.
    pub fn is_valid_var_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let first = name.chars().next().unwrap();
        if !first.is_alphabetic() && first != '_' {
            return false;
        }
        name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Extract environment variable references from a bash command string.
    pub fn extract_env_vars(cmd: &str) -> Vec<String> {
        let mut vars = Vec::new();
        let chars: Vec<char> = cmd.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if next == '{' {
                    // ${VAR} form
                    if let Some(end) = cmd[i + 2..].find('}') {
                        let var_name = &cmd[i + 2..i + 2 + end];
                        // Strip any parameter expansion operators
                        let var_name = var_name
                            .split(|c: char| c == ':' || c == '-' || c == '+' || c == '=')
                            .next()
                            .unwrap_or(var_name);
                        if is_valid_var_name(var_name) {
                            vars.push(var_name.to_string());
                        }
                        i += 3 + end;
                        continue;
                    }
                } else if next.is_alphabetic() || next == '_' {
                    // $VAR form
                    let start = i + 1;
                    let mut end = start;
                    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                        end += 1;
                    }
                    let var_name: String = chars[start..end].iter().collect();
                    if is_valid_var_name(&var_name) {
                        vars.push(var_name);
                    }
                    i = end;
                    continue;
                }
            }
            i += 1;
        }

        vars
    }
}

/// Command mapping between bash and PowerShell built-in commands.
pub mod command_map {
    use std::collections::HashMap;

    /// Returns a mapping of common bash commands to their PowerShell equivalents.
    pub fn bash_to_powershell() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        map.insert("echo", "Write-Output");
        map.insert("cat", "Get-Content");
        map.insert("ls", "Get-ChildItem");
        map.insert("cp", "Copy-Item");
        map.insert("mv", "Move-Item");
        map.insert("rm", "Remove-Item");
        map.insert("mkdir", "New-Item -ItemType Directory -Path");
        map.insert("rmdir", "Remove-Item -Recurse");
        map.insert("pwd", "Get-Location");
        map.insert("cd", "Set-Location");
        map.insert("grep", "Select-String");
        map.insert("find", "Get-ChildItem -Recurse");
        map.insert("sort", "Sort-Object");
        map.insert("head", "Select-Object -First");
        map.insert("tail", "Select-Object -Last");
        map.insert("wc", "Measure-Object");
        map.insert("touch", "New-Item -ItemType File -Path");
        map.insert("chmod", "# chmod not applicable on Windows");
        map.insert("chown", "# chown not applicable on Windows");
        map.insert("which", "Get-Command");
        map.insert("whoami", "$env:USERNAME");
        map.insert("hostname", "$env:COMPUTERNAME");
        map.insert("date", "Get-Date");
        map.insert("sleep", "Start-Sleep -Seconds");
        map.insert("kill", "Stop-Process -Id");
        map.insert("ps", "Get-Process");
        map.insert("env", "Get-ChildItem Env:");
        map.insert("export", "$env:");
        map.insert("unset", "Remove-Item Env:");
        map.insert("curl", "Invoke-WebRequest");
        map.insert("wget", "Invoke-WebRequest -OutFile");
        map.insert("tar", "Expand-Archive");
        map.insert("zip", "Compress-Archive");
        map.insert("unzip", "Expand-Archive");
        map.insert("diff", "Compare-Object");
        map.insert("tee", "Tee-Object");
        map.insert("true", "$true");
        map.insert("false", "$false");
        map.insert("test", "Test-Path");
        map
    }

    /// Returns a mapping of bash operators to PowerShell operators.
    pub fn bash_operators_to_powershell() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        map.insert("&&", "&&"); // PowerShell 7+ supports && natively
        map.insert("||", "||"); // PowerShell 7+ supports || natively
        map.insert("|", "|");
        map.insert(">", ">"); // same in PS
        map.insert(">>", ">>"); // same in PS
        map.insert("2>&1", "*>&1");
        map.insert("/dev/null", "$null");
        map
    }
}

/// PowerShell conversion utilities.
///
/// This module provides functions for converting bash commands and scripts
/// to their PowerShell equivalents, handling environment variables, special
/// variables, subshells, and command substitutions.
pub mod powershell {
    use super::command_map;

    /// Convert a bash command string to its PowerShell equivalent.
    ///
    /// This function handles:
    /// - Command name translation (echo -> Write-Output, etc.)
    /// - Environment variable conversion ($HOME -> $env:HOME)
    /// - Special variable conversion ($? -> $LASTEXITCODE, $$ -> $PID)
    /// - Subshell preservation ($(cmd) is left as-is for PowerShell)
    /// - String literal preservation (dollar signs in single quotes are untouched)
    /// - Operator translation (2>&1 -> *>&1, /dev/null -> $null, etc.)
    ///
    /// # Arguments
    ///
    /// * `bash_cmd` - The bash command string to convert
    ///
    /// # Returns
    ///
    /// A String containing the PowerShell equivalent of the bash command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bounty_challenge::shell::powershell;
    ///
    /// assert_eq!(
    ///     powershell::from_bash("echo $HOME"),
    ///     "Write-Output $env:HOME"
    /// );
    ///
    /// // Special variables are mapped correctly
    /// assert_eq!(
    ///     powershell::from_bash("echo $?"),
    ///     "Write-Output $LASTEXITCODE"
    /// );
    ///
    /// // Subshells are preserved
    /// assert!(powershell::from_bash("echo $(whoami)").contains("$("));
    /// ```
    pub fn from_bash(bash_cmd: &str) -> String {
        let mut result = bash_cmd.to_string();

        // Step 1: Convert common bash commands to PowerShell equivalents
        let cmd_map = command_map::bash_to_powershell();
        let tokens = super::bash::tokenize(&result);
        if let Some(first_token) = tokens.first() {
            // Strip any leading path components (e.g., /usr/bin/echo -> echo)
            let cmd_name = first_token
                .rsplit('/')
                .next()
                .unwrap_or(first_token);
            if let Some(ps_cmd) = cmd_map.get(cmd_name) {
                result = result.replacen(first_token, ps_cmd, 1);
            }
        }

        // Step 2: Convert bash operators to PowerShell equivalents
        // Only replace operators outside of quoted strings.
        let op_map = command_map::bash_operators_to_powershell();
        for (bash_op, ps_op) in &op_map {
            if *bash_op != "|" && bash_op != ps_op {
                result = replace_outside_quotes(&result, bash_op, ps_op);
            }
        }

        // Step 3: Convert environment variables with context awareness
        // Only replace $VAR_NAME patterns that are actual environment variable
        // references. Do NOT replace:
        //   - Subshell execution: $(...)
        //   - Special variables: $?, $$, $!, $#, $@, $*, $0-$9
        //   - Dollar signs inside single-quoted strings
        //   - Brace-delimited variables: ${VAR} (handled separately)
        //   - Literal dollar signs followed by non-variable characters
        result = convert_dollar_signs(&result);

        result
    }

    /// Context-aware conversion of dollar-sign expressions from bash to PowerShell.
    ///
    /// Walks through the string character by character and determines the
    /// correct PowerShell equivalent for each `$` occurrence based on what
    /// follows it.
    fn convert_dollar_signs(input: &str) -> String {
        let mut result = String::with_capacity(input.len() + 32);
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        let mut i = 0;
        let mut in_single_quote = false;

        while i < len {
            let c = chars[i];

            // Track single-quote state: dollar signs inside single quotes
            // are literal in bash, so leave them untouched.
            if c == '\'' {
                in_single_quote = !in_single_quote;
                result.push(c);
                i += 1;
                continue;
            }

            // Inside single quotes, everything is literal
            if in_single_quote {
                result.push(c);
                i += 1;
                continue;
            }

            if c == '$' && i + 1 < len {
                let next = chars[i + 1];

                match next {
                    // $( ... ) -- subshell / command substitution
                    // PowerShell also uses $() for subexpressions, so leave as-is
                    '(' => {
                        result.push('$');
                        // We push '$' and let the rest of the string flow through
                        i += 1;
                        continue;
                    }

                    // ${ ... } -- brace-delimited variable
                    '{' => {
                        if let Some(close_pos) = chars[i + 2..].iter().position(|&ch| ch == '}') {
                            let var_name: String =
                                chars[i + 2..i + 2 + close_pos].iter().collect();
                            // Strip parameter expansion operators for the var name
                            let base_var = var_name
                                .split(|c: char| c == ':' || c == '-' || c == '+' || c == '=')
                                .next()
                                .unwrap_or(&var_name);
                            if super::bash::is_valid_var_name(base_var) {
                                result.push_str(&format!("$env:{}", base_var));
                            } else {
                                // Not a valid var name, keep original
                                result.push('$');
                                result.push('{');
                                result.push_str(&var_name);
                                result.push('}');
                            }
                            i += 3 + close_pos; // skip ${ ... }
                            continue;
                        }
                        // No closing brace found, treat as literal
                        result.push('$');
                        i += 1;
                        continue;
                    }

                    // $? -- last exit status -> $LASTEXITCODE in PowerShell
                    '?' => {
                        result.push_str("$LASTEXITCODE");
                        i += 2;
                        continue;
                    }

                    // $$ -- current process ID -> $PID in PowerShell
                    '$' => {
                        result.push_str("$PID");
                        i += 2;
                        continue;
                    }

                    // $! -- last background process ID (no direct PowerShell equivalent)
                    '!' => {
                        result.push_str("<# $! not supported #>");
                        i += 2;
                        continue;
                    }

                    // $# -- number of positional parameters -> $args.Count
                    '#' => {
                        result.push_str("$args.Count");
                        i += 2;
                        continue;
                    }

                    // $@ or $* -- all positional parameters -> $args
                    '@' | '*' => {
                        result.push_str("$args");
                        i += 2;
                        continue;
                    }

                    // $0-$9 -- positional parameters -> $args[N-1] (or $MyInvocation for $0)
                    d if d.is_ascii_digit() => {
                        if d == '0' {
                            result.push_str("$MyInvocation.MyCommand.Name");
                        } else {
                            result.push_str(&format!("$args[{}]", (d as u8 - b'1')));
                        }
                        i += 2;
                        continue;
                    }

                    // $_ -- common in bash (though rare) and also valid in PS
                    // Guard: either at end of string or next char is not alphanumeric
                    '_' if i + 2 >= len || !chars[i + 2].is_alphanumeric() => {
                        result.push_str("$_");
                        i += 2;
                        continue;
                    }

                    // $VARNAME -- environment variable reference
                    ch if ch.is_alphabetic() || ch == '_' => {
                        let start = i + 1;
                        let mut end = start;
                        while end < len && (chars[end].is_alphanumeric() || chars[end] == '_') {
                            end += 1;
                        }
                        let var_name: String = chars[start..end].iter().collect();
                        result.push_str(&format!("$env:{}", var_name));
                        i = end;
                        continue;
                    }

                    // Any other character after $ -- treat $ as literal
                    _ => {
                        result.push('$');
                        i += 1;
                        continue;
                    }
                }
            } else if c == '$' && i + 1 == len {
                // Trailing $ at end of string -- literal
                result.push('$');
                i += 1;
                continue;
            } else {
                result.push(c);
                i += 1;
            }
        }

        result
    }

    /// Replace occurrences of `from` with `to` only when they appear outside of
    /// single-quoted or double-quoted strings.  This prevents operator
    /// replacement from corrupting string literals like `"a && b"`.
    fn replace_outside_quotes(input: &str, from: &str, to: &str) -> String {
        let mut result = String::with_capacity(input.len() + 32);
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let chars: Vec<char> = input.chars().collect();
        let from_chars: Vec<char> = from.chars().collect();
        let len = chars.len();
        let flen = from_chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                result.push(c);
                i += 1;
                continue;
            }
            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                result.push(c);
                i += 1;
                continue;
            }

            if !in_single_quote && !in_double_quote && i + flen <= len {
                if chars[i..i + flen] == from_chars[..] {
                    result.push_str(to);
                    i += flen;
                    continue;
                }
            }

            result.push(c);
            i += 1;
        }

        result
    }

    /// Convert a PowerShell command to bash equivalent.
    pub fn to_bash(ps_cmd: &str) -> String {
        let mut result = ps_cmd.to_string();

        // Convert PowerShell environment variables to bash
        // $env:VAR -> $VAR
        let prefix = "$env:";
        let mut new_result = String::with_capacity(result.len());
        let bytes = result.as_bytes();
        let prefix_bytes = prefix.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if i + prefix_bytes.len() <= bytes.len()
                && &bytes[i..i + prefix_bytes.len()] == prefix_bytes
            {
                new_result.push('$');
                i += prefix_bytes.len();
                // Collect the variable name (ASCII-safe: alphanumeric and _)
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    new_result.push(bytes[i] as char);
                    i += 1;
                }
            } else {
                // Safe: $env: prefix is ASCII, so non-prefix bytes keep
                // their original encoding.  We re-derive the char properly.
                let ch = result[i..].chars().next().unwrap();
                new_result.push(ch);
                i += ch.len_utf8();
            }
        }
        result = new_result;

        // Convert PowerShell commands back to bash
        result = result.replace("Write-Output", "echo");
        result = result.replace("Get-Content", "cat");
        result = result.replace("Get-ChildItem", "ls");
        result = result.replace("Copy-Item", "cp");
        result = result.replace("Move-Item", "mv");
        result = result.replace("Remove-Item", "rm");
        result = result.replace("Get-Location", "pwd");
        result = result.replace("Set-Location", "cd");

        // Convert special variables
        result = result.replace("$LASTEXITCODE", "$?");
        result = result.replace("$PID", "$$");

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ShellType tests =====

    #[test]
    fn test_shell_type_is_posix() {
        assert!(ShellType::Bash.is_posix());
        assert!(ShellType::Zsh.is_posix());
        assert!(!ShellType::PowerShell.is_posix());
        assert!(!ShellType::Cmd.is_posix());
        assert!(!ShellType::Fish.is_posix());
    }

    #[test]
    fn test_shell_type_is_windows() {
        assert!(ShellType::PowerShell.is_windows());
        assert!(ShellType::Cmd.is_windows());
        assert!(!ShellType::Bash.is_windows());
        assert!(!ShellType::Zsh.is_windows());
    }

    // ===== bash::escape tests =====

    #[test]
    fn test_bash_escape_simple() {
        assert_eq!(bash::escape("hello"), "'hello'");
    }

    #[test]
    fn test_bash_escape_single_quotes() {
        assert_eq!(bash::escape("it's"), "'it'\\''s'");
    }

    // ===== bash::tokenize tests =====

    #[test]
    fn test_tokenize_simple() {
        let tokens = bash::tokenize("echo hello world");
        assert_eq!(tokens, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_tokenize_quoted() {
        let tokens = bash::tokenize("echo \"hello world\"");
        assert_eq!(tokens, vec!["echo", "\"hello world\""]);
    }

    #[test]
    fn test_tokenize_single_quoted() {
        let tokens = bash::tokenize("echo 'hello world'");
        assert_eq!(tokens, vec!["echo", "'hello world'"]);
    }

    // ===== bash::is_valid_var_name tests =====

    #[test]
    fn test_valid_var_names() {
        assert!(bash::is_valid_var_name("HOME"));
        assert!(bash::is_valid_var_name("_private"));
        assert!(bash::is_valid_var_name("var123"));
        assert!(!bash::is_valid_var_name("123var"));
        assert!(!bash::is_valid_var_name(""));
        assert!(!bash::is_valid_var_name("var-name"));
    }

    // ===== bash::extract_env_vars tests =====

    #[test]
    fn test_extract_env_vars() {
        let vars = bash::extract_env_vars("echo $HOME and $PATH");
        assert!(vars.contains(&"HOME".to_string()));
        assert!(vars.contains(&"PATH".to_string()));
    }

    #[test]
    fn test_extract_env_vars_braces() {
        let vars = bash::extract_env_vars("echo ${HOME}/.config");
        assert!(vars.contains(&"HOME".to_string()));
    }

    // ===== powershell::from_bash tests =====

    #[test]
    fn test_from_bash_simple_echo() {
        let result = powershell::from_bash("echo hello");
        assert_eq!(result, "Write-Output hello");
    }

    #[test]
    fn test_from_bash_env_var() {
        let result = powershell::from_bash("echo $HOME");
        assert_eq!(result, "Write-Output $env:HOME");
    }

    #[test]
    fn test_from_bash_multiple_env_vars() {
        let result = powershell::from_bash("echo $HOME $PATH $USER");
        assert_eq!(result, "Write-Output $env:HOME $env:PATH $env:USER");
    }

    #[test]
    fn test_from_bash_subshell_preserved() {
        // $(cmd) should be preserved, not converted to $env:(cmd)
        let result = powershell::from_bash("echo $(whoami)");
        assert!(
            result.contains("$("),
            "Subshell $() should be preserved, got: {}",
            result
        );
        assert!(
            !result.contains("$env:("),
            "Subshell should NOT be converted to $env:(, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_exit_status() {
        // $? should become $LASTEXITCODE
        let result = powershell::from_bash("echo $?");
        assert_eq!(result, "Write-Output $LASTEXITCODE");
    }

    #[test]
    fn test_from_bash_process_id() {
        // $$ should become $PID
        let result = powershell::from_bash("echo $$");
        assert_eq!(result, "Write-Output $PID");
    }

    #[test]
    fn test_from_bash_dollar_in_single_quotes() {
        // Dollar signs inside single quotes are literal in bash
        let result = powershell::from_bash("echo '$HOME'");
        assert!(
            result.contains("'$HOME'"),
            "Dollar sign in single quotes should be literal, got: {}",
            result
        );
        assert!(
            !result.contains("$env:HOME"),
            "Should not convert vars inside single quotes, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_dollar_followed_by_digit() {
        // $1, $2, etc. are positional parameters, not env vars
        let result = powershell::from_bash("echo $1");
        assert!(
            !result.contains("$env:1"),
            "Positional parameter $1 should NOT become $env:1, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_dollar_literal_amount() {
        // "$50" should not become "$env:50"
        let result = powershell::from_bash("echo \"$50\"");
        assert!(
            !result.contains("$env:50"),
            "Dollar amount $50 should not become $env:50, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_hash_var() {
        // $# should become $args.Count
        let result = powershell::from_bash("echo $#");
        assert!(
            result.contains("$args.Count"),
            "$# should become $args.Count, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_at_var() {
        // $@ should become $args
        let result = powershell::from_bash("echo $@");
        assert!(
            result.contains("$args"),
            "$@ should become $args, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_brace_variable() {
        // ${HOME} should become $env:HOME
        let result = powershell::from_bash("echo ${HOME}");
        assert!(
            result.contains("$env:HOME"),
            "${HOME} should become $env:HOME, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_complex_command() {
        // A more complex command with multiple dollar-sign patterns
        let result = powershell::from_bash("echo $HOME $(date) $? '$$'");
        assert!(result.contains("$env:HOME"), "Should convert $HOME");
        assert!(result.contains("$(date)") || result.contains("$("), "Should preserve $(date)");
        assert!(
            result.contains("$LASTEXITCODE"),
            "Should convert $? to $LASTEXITCODE"
        );
    }

    #[test]
    fn test_from_bash_trailing_dollar() {
        // A trailing $ should be kept as literal
        let result = powershell::from_bash("echo cost$");
        assert!(
            result.contains("cost$"),
            "Trailing $ should be literal, got: {}",
            result
        );
    }

    // ===== powershell::to_bash tests =====

    #[test]
    fn test_to_bash_env_var() {
        let result = powershell::to_bash("Write-Output $env:HOME");
        assert!(result.contains("echo $HOME"));
    }

    #[test]
    fn test_to_bash_exit_code() {
        let result = powershell::to_bash("$LASTEXITCODE");
        assert_eq!(result, "$?");
    }

    // ===== Issue #1: && should use PowerShell 7 native && operator =====

    #[test]
    fn test_from_bash_and_operator_preserved() {
        // && should remain && for PowerShell 7+ (not become ;)
        let result = powershell::from_bash("mkdir foo && cd foo");
        assert!(
            result.contains("&&"),
            "&& should be preserved for PowerShell 7+, got: {}",
            result
        );
        assert!(
            !result.contains("; cd"),
            "&& should NOT become ;, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_or_operator_preserved() {
        // || should remain || for PowerShell 7+
        let result = powershell::from_bash("cmd1 || cmd2");
        assert!(
            result.contains("||"),
            "|| should be preserved for PowerShell 7+, got: {}",
            result
        );
    }

    // ===== Issue #2: Operator replacement should not corrupt quoted strings =====

    #[test]
    fn test_from_bash_operator_inside_quotes_untouched() {
        // Operators inside quoted strings must not be replaced
        let result = powershell::from_bash("echo \"a && b\"");
        assert!(
            result.contains("\"a && b\""),
            "Operators inside double quotes should be untouched, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_operator_inside_single_quotes_untouched() {
        let result = powershell::from_bash("echo '2>&1'");
        assert!(
            result.contains("'2>&1'"),
            "Operators inside single quotes should be untouched, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_devnull_inside_quotes_untouched() {
        let result = powershell::from_bash("echo \"/dev/null\"");
        assert!(
            result.contains("\"/dev/null\""),
            "/dev/null inside quotes should be untouched, got: {}",
            result
        );
    }

    // ===== Issue #3: $! should NOT map to $PID =====

    #[test]
    fn test_from_bash_bang_not_pid() {
        // $! is last background PID in bash; no direct PS equivalent
        let result = powershell::from_bash("echo $!");
        assert!(
            !result.contains("$PID"),
            "$! should NOT map to $PID, got: {}",
            result
        );
        assert!(
            result.contains("<# $! not supported #>"),
            "$! should map to a placeholder comment, got: {}",
            result
        );
    }

    // ===== Issue #4: $_ at end-of-string should not become $env:_ =====

    #[test]
    fn test_from_bash_dollar_underscore_end_of_string() {
        // When $_ is at the end of the string, it should stay as $_
        let result = powershell::from_bash("echo $_");
        assert!(
            result.contains("$_"),
            "$_ at end of string should remain $_, got: {}",
            result
        );
        assert!(
            !result.contains("$env:_"),
            "$_ should NOT become $env:_, got: {}",
            result
        );
    }

    #[test]
    fn test_from_bash_dollar_underscore_mid_string() {
        // $_ followed by a space should still be $_
        let result = powershell::from_bash("echo $_ foo");
        assert!(
            result.contains("$_"),
            "$_ followed by space should remain $_, got: {}",
            result
        );
        assert!(
            !result.contains("$env:_"),
            "$_ should NOT become $env:_, got: {}",
            result
        );
    }

    // ===== Issue #5: to_bash should not have O(n^2) allocation =====

    #[test]
    fn test_to_bash_large_input_no_regression() {
        // Verify correctness on a moderately large input (the fix is about
        // performance, but we test correctness here).
        let large = "$env:HOME ".repeat(500);
        let result = powershell::to_bash(&large);
        assert!(result.contains("$HOME"), "Should still convert $env:HOME");
        assert!(!result.contains("$env:"), "Should not have leftover $env:");
    }

    #[test]
    fn test_to_bash_unicode_safe() {
        // Ensure the byte-level scanning in to_bash handles non-ASCII
        let result = powershell::to_bash("Write-Output $env:HOME \u{1F600}");
        assert!(result.contains("$HOME"));
        assert!(result.contains("\u{1F600}"));
    }

    // ===== command_map tests =====

    #[test]
    fn test_command_map_has_common_commands() {
        let map = command_map::bash_to_powershell();
        assert_eq!(map.get("echo"), Some(&"Write-Output"));
        assert_eq!(map.get("cat"), Some(&"Get-Content"));
        assert_eq!(map.get("ls"), Some(&"Get-ChildItem"));
        assert_eq!(map.get("pwd"), Some(&"Get-Location"));
    }

    #[test]
    fn test_operator_map_uses_native_pipeline_operators() {
        let map = command_map::bash_operators_to_powershell();
        assert_eq!(
            map.get("&&"),
            Some(&"&&"),
            "&&  should map to && for PowerShell 7+"
        );
        assert_eq!(
            map.get("||"),
            Some(&"||"),
            "|| should map to || for PowerShell 7+"
        );
    }
}
