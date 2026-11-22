use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["release", flags @ ..] => release(flags),
        _ => Err("usage: cargo release [--dry-run] [--yes]".to_string()),
    };
    if let Err(error) = result {
        eprintln!("error: {}", error);
        exit(1);
    }
}

fn release(flags: &[&str]) -> Result<(), String> {
    let dry_run = flags.contains(&"--dry-run");
    let yes = flags.contains(&"--yes");
    if let Some(flag) = flags.iter().find(|f| !["--dry-run", "--yes"].contains(f)) {
        return Err(format!("unknown flag {}", flag));
    }

    let root = root();
    let tag = format!("v{}", version(&root)?);

    if output(&root, "git", &["branch", "--show-current"])? != "main" {
        return Err("releases are cut from main".into());
    }
    if !output(&root, "git", &["status", "--porcelain"])?.is_empty() {
        return Err("the working tree has uncommitted changes".into());
    }
    run(&root, "git", &["fetch", "--quiet", "--tags", "origin", "main"])?;
    if output(&root, "git", &["rev-parse", "HEAD"])? != output(&root, "git", &["rev-parse", "origin/main"])? {
        return Err("main isn't up to date with origin/main".into());
    }
    if output(&root, "git", &["tag", "--list", &tag])? == tag {
        return Err(format!("{} is already tagged", tag));
    }
    if !output(&root, "git", &["ls-remote", "--tags", "origin", &format!("refs/tags/{}", tag)])?.is_empty() {
        return Err(format!("{} is already tagged on origin", tag));
    }

    run(&root, "cargo", &["publish", "--locked", "--dry-run"])?;
    if dry_run {
        println!("Dry run passed; `cargo release` would publish {} and push its tag.", tag);
        return Ok(());
    }
    if !yes && !confirm(&format!("Publish {} to crates.io and push its tag?", tag))? {
        return Err("cancelled".into());
    }

    run(&root, "git", &["tag", "--annotate", &tag, "--message", &tag])?;
    if let Err(error) = run(&root, "cargo", &["publish", "--locked"]) {
        let _ = run(&root, "git", &["tag", "--delete", &tag]);
        return Err(error);
    }
    run(&root, "git", &["push", "origin", &tag]).map_err(|error| {
        format!("{} was published, but pushing its tag failed ({}); retry with `git push origin {}`", tag, error, tag)
    })?;

    println!("Released {}. Publish the draft GitHub release once its binaries are attached.", tag);
    Ok(())
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

fn version(root: &Path) -> Result<String, String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|error| format!("unable to read Cargo.toml: {}", error))?;
    manifest
        .lines()
        .skip_while(|line| line.trim() != "[package]")
        .find_map(|line| line.strip_prefix("version = "))
        .map(|version| version.trim_matches('"').to_string())
        .ok_or_else(|| "no package version in Cargo.toml".to_string())
}

fn confirm(question: &str) -> Result<bool, String> {
    print!("{} [y/N] ", question);
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| error.to_string())?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|error| format!("unable to run {}: {}", program, error))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("`{} {}` failed", program, args.join(" "))),
    }
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("unable to run {}: {}", program, error))?;
    match output.status.success() {
        true => Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()),
        false => Err(format!(
            "`{} {}` failed: {}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}
