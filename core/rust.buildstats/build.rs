use std::process::Command;

type Error = Box<dyn std::error::Error + Send + Sync>;

fn set_stats() -> Result<(), Error> {
    // Check for git and existence of .git
    let is_git_installed_and_is_repo = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()?;

    if is_git_installed_and_is_repo.status.success() {
        // Git commit hash
        let git_commit_hash = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()?;

        let git_commit_hash = String::from_utf8(git_commit_hash.stdout)?.replace('\n', "");

        // Git commit message
        let git_commit_message = Command::new("git")
            .args(["log", "-1", "--pretty=%B"])
            .output()?;

        let git_commit_message = String::from_utf8(git_commit_message.stdout)?.replace('\n', "");

        // git repo url
        let git_repo = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .output()?;

        let git_repo = String::from_utf8(git_repo.stdout)?.replace('\n', "");

        println!(
            "cargo:rustc-env=__BUILDSTATS__GIT_COMMIT_HASH={}",
            git_commit_hash
        );
        println!(
            "cargo:rustc-env=__BUILDSTATS__GIT_COMMIT_MESSAGE={}",
            git_commit_message
        );
        println!("cargo:rustc-env=__BUILDSTATS__GIT_REPO={}", git_repo);

        // Set rerun if changed to .git/HEAD
        println!("cargo:rerun-if-changed=.git/HEAD");
    } else {
        println!("cargo:rustc-env=__BUILDSTATS__GIT_COMMIT_HASH=Unknown");
        println!("cargo:rustc-env=__BUILDSTATS__GIT_COMMIT_MESSAGE=Unknown");
        println!("cargo:rustc-env=__BUILDSTATS__GIT_REPO=Unknown");
    }

    // Lastly, get the cpu model
    let proc_cpuinfo_exists = std::path::Path::new("/proc/cpuinfo").exists();

    if proc_cpuinfo_exists {
        let cpu_model = Command::new("cat").args(["/proc/cpuinfo"]).output()?;

        let cpu_model = String::from_utf8(cpu_model.stdout)?;

        let mut model_found = false;
        for line in cpu_model.lines().take(13) {
            if line.starts_with("model name") {
                let model = line.split(':').nth(1).unwrap().trim();
                println!("cargo:rustc-env=__BUILDSTATS__CPU_MODEL={}", model);
                model_found = true;
                break;
            }
        }

        if !model_found {
            println!("cargo:rustc-env=C__BUILDSTATS__PU_MODEL=Unknown CPU");
        }

        println!("cargo:rerun-if-changed=/proc/cpuinfo");
    } else {
        println!("cargo:rustc-env=__BUILDSTATS__CPU_MODEL=Unknown CPU");
    }

    // rustc version
    let rustc_version = Command::new("rustc").args(["-V"]).output()?;

    let rustc_version = String::from_utf8(rustc_version.stdout)?.replace('\n', "");

    // Strip out extra data from rustc version by splitting by ( and taking the first part
    // E.g. rustc 1.79.0-nightly (ab5bda1aa 2024-04-08) becomes rustc 1.79.0-nightly
    let rustc_version = rustc_version.split('(').next().unwrap().to_string();

    println!(
        "cargo:rustc-env=__BUILDSTATS__RUSTC_VERSION={}",
        rustc_version
    );

    // Get profile
    let profile = std::env::var("PROFILE").unwrap_or("unknown".to_string());

    println!("cargo:rustc-env=__BUILDSTATS__CARGO_PROFILE={}", profile);

    Ok(())
}

fn main() -> Result<(), Error> {
    // CI means we probably dont want to do extensive checks
    if std::env::var("CI_BUILD").unwrap_or_default() == "true" {
        println!("cargo:rustc-env=__BUILDSTATS__GIT_COMMIT_HASH=Unknown");
        println!("cargo:rustc-env=__BUILDSTATS__GIT_COMMIT_MESSAGE=Unknown");
        println!("cargo:rustc-env=__BUILDSTATS__GIT_REPO=Unknown");
        println!("cargo:rustc-env=__BUILDSTATS__CPU_MODEL=CI");
        println!("cargo:rustc-env=__BUILDSTATS__RUSTC_VERSION=CI");
        println!(
            "cargo:rustc-env=__BUILDSTATS__CARGO_PROFILE={}",
            std::env::var("PROFILE").unwrap_or("unknown".to_string())
        );
        return Ok(());
    }

    set_stats()?;

    Ok(())
}
