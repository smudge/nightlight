use std::process::Command;

pub(crate) trait Runner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String>;
}

pub(crate) struct System;

impl Runner for System {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|error| format!("Unable to run {}: {}", program, error))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = match stderr.trim() {
                "" => stdout.trim(),
                stderr => stderr,
            };
            Err(format!("{} {} failed: {}", program, args.join(" "), detail))
        }
    }
}
