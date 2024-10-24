use serde::Deserialize;
use std::{
    fs::{self, create_dir},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};
use tempfile::{tempdir, tempdir_in, TempDir};

use clap::Parser;

#[derive(Clone, Debug, Deserialize)]
struct Project {
    url: String,
    name: String,
    replace_prefix: String,
    yaml_path: Option<String>,
    test_cmd: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Config {
    projects: Vec<Project>,
}

#[derive(Clone, Parser)]
struct Cli {
    /// Config
    config: PathBuf,
    author_name: String,
    author_email: String,
    update: String,

    /// Root directory, tmp if not given
    #[arg(long)]
    root_dir: Option<PathBuf>,

    /// Don't exit on single failure
    #[arg(long)]
    no_exit_on_error: bool,

    /// Don't emit stdout while running commands
    #[arg(long)]
    no_stdout: bool,
}

fn tmp_dir(root_dir: &Option<PathBuf>) -> TempDir {
    if let Some(ref root_dir) = root_dir {
        tempdir_in(root_dir).unwrap()
    } else {
        tempdir().unwrap()
    }
}

fn run(cli: &Cli, command: &mut Command) -> io::Result<bool> {
    if !cli.no_stdout {
        command.stdout(Stdio::piped());

        let mut child = command.spawn()?;

        // Ensure the child's stdout can be captured
        if let Some(stdout) = child.stdout.take() {
            // Create a buffered reader to process the stdout line-by-line
            let reader = BufReader::new(stdout);

            // Read lines from the command's stdout as they are produced
            for line in reader.lines() {
                // Print each line to the current program's stdout
                println!("{}", line?);
            }
        }

        // Wait for the child process to finish
        let status = child.wait()?;
        let success = status.success();
        if success {
            println!("[-] success!");
        }
        Ok(success)
    } else {
        let output = command.output().unwrap();
        let success = output.status.success();
        if !success {
            let stdout = String::from_utf8(output.stdout).unwrap();
            println!("stdout: {}", stdout);
            let stderr = String::from_utf8(output.stderr).unwrap();
            println!("stderr: {}", stderr);
        }
        if success {
            println!("[-] success!");
        }
        Ok(success)
    }
}

fn main() {
    let args = Cli::parse();

    let toml_content = fs::read_to_string(&args.config).unwrap();
    let config: Config = toml::from_str(&toml_content).unwrap();

    if let Some(root_dir) = &args.root_dir {
        if !fs::exists(root_dir).unwrap() {
            create_dir(root_dir).unwrap();
        }
    }

    for Project {
        url,
        name,
        replace_prefix,
        yaml_path,
        test_cmd,
    } in config.projects
    {
        let tmp_dir = tmp_dir(&args.root_dir);
        // persist
        let tmp_dir = tmp_dir.into_path();
        // let tmp_dir = tmp_dir.path();
        println!("[-] Cloning into {:?}", tmp_dir);

        let git_args = vec!["clone".to_string(), url.to_string()];

        let mut cmd = Command::new("git");
        cmd.args(git_args).current_dir(&tmp_dir);
        if !run(&args, &mut cmd).unwrap() && !args.no_exit_on_error {
            break;
        }

        let proj_dir = tmp_dir.join(name);

        // Test before
        if let Some(test_cmd) = &test_cmd {
            run_extra_cmd(&args, &test_cmd, &proj_dir);
        }

        let yaml_path = yaml_path.unwrap_or(".gitlab.yml".to_string());
        let yaml_path = proj_dir.join(yaml_path);
        let contents = fs::read_to_string(&yaml_path).unwrap();

        let modified_lines: Vec<String> = contents
            .lines()
            .map(|line| {
                if let Some(pos) = line.find(&replace_prefix) {
                    // Create a new line with the replacement
                    let new_line =
                        format!("{} {}", &line[..pos + replace_prefix.len()], args.update);
                    new_line
                } else {
                    line.to_string()
                }
            })
            .collect();
        let mut new_contents = modified_lines.join("\n");
        new_contents.push_str("\n");

        fs::write(&yaml_path, new_contents).unwrap();

        // Checkout, commit, push
        let mut cmd = Command::new("git");
        let branch_ver = args.update.replace(':', "-").replace('.', "-");
        let branch_name = format!("update-to-{branch_ver}");
        cmd.args(["checkout", "-b", &branch_name])
            .current_dir(&proj_dir);
        if !run(&args, &mut cmd).unwrap() && !args.no_exit_on_error {
            continue;
        }

        let mut cmd = Command::new("git");
        cmd.args(["add", &yaml_path.to_str().unwrap()])
            .current_dir(&proj_dir);
        if !run(&args, &mut cmd).unwrap() && !args.no_exit_on_error {
            continue;
        }

        let mut cmd = Command::new("git");
        cmd.args([
            "commit",
            "-m",
            &format!("Update to {}", args.update),
            "--author",
            &format!("{} <{}>", &args.author_name, &args.author_email),
        ])
        .env("GIT_COMMITTER_NAME", &args.author_name)
        .env("GIT_COMMITTER_EMAIL", &args.author_email)
        .current_dir(&proj_dir);
        if !run(&args, &mut cmd).unwrap() && !args.no_exit_on_error {
            continue;
        }

        // Test before
        if let Some(test_cmd) = &test_cmd {
            run_extra_cmd(&args, &test_cmd, &proj_dir);
        }

        let mut cmd = Command::new("git");
        cmd.args(["push", "origin", &branch_name])
            .current_dir(&proj_dir);
        if !run(&args, &mut cmd).unwrap() && !args.no_exit_on_error {
            continue;
        }
    }
}

fn run_extra_cmd(args: &Cli, test_extra_cmd: &str, proj_dir: &PathBuf) {
    println!("[-] running extra cmd: {test_extra_cmd}");
    let split: Vec<&str> = test_extra_cmd.split_whitespace().collect();
    let mut cmd = Command::new(&split[0]);
    cmd.args(&split[1..]).current_dir(proj_dir);
    if !run(&args, &mut cmd).unwrap() && !args.no_exit_on_error {
        return;
    }
}
