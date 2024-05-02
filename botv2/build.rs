use std::process::Command;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Build src/modules/mod.rs based on the folder listing of src/modules
fn autogen_modules_mod_rs() -> Result<(), Error> {
    const MODULE_TEMPLATE: &str = r#"
// Auto-generated by build.rs
{module_use_list}

/// List of modules available. Not all may be enabled
pub fn modules() -> Vec<crate::silverpelt::Module> {
    vec![
        {module_func_list}
    ]
}
    "#;

    let mut module_list = Vec::new();

    let folder_list = std::fs::read_dir("src/modules")?;

    for folder in folder_list {
        let folder = folder?;

        if !folder.file_type().unwrap().is_dir() {
            continue;
        }

        // Check that a mod.rs file exists in the folder
        let mod_rs_path = folder.path().join("mod.rs");

        // A TOCTOU here isn't important as this is just a one-of build script
        if !mod_rs_path.exists() {
            continue;
        }

        let folder_name = folder.file_name().into_string().unwrap();

        module_list.push(folder_name);
    }

    module_list.sort();

    // Move root to bottom
    if let Some(root_index) = module_list.iter().position(|x| x == "root") {
        let root = module_list.remove(root_index);
        module_list.push(root);
    }

    // Construct module_uses_list
    let mut module_use_list = Vec::new();

    for module in &module_list {
        module_use_list.push(format!("pub mod {};", module));
    }

    let module_use_list = module_use_list.join("\n");

    // Construct module_list
    let mut module_dat_list = Vec::new();

    for module in &module_list {
        module_dat_list.push(format!("{}::module(),", module));
    }

    let module_func_list = module_dat_list.join("\n        ");

    let mut module_ids_list = Vec::new();

    for module in &module_list {
        module_ids_list.push(format!("\"{}\",", module));
    }

    let module_list_final = MODULE_TEMPLATE
        .replace("{module_use_list}", &module_use_list)
        .replace("{module_func_list}", &module_func_list)
        .replace("{module_ids_list}", &module_ids_list.join("\n        ")); // Not used currently but may be used in the future

    let module_list_final = module_list_final.trim().to_string();

    std::fs::write("src/modules/mod.rs", module_list_final)?;

    Ok(())
}

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

        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash);
        println!("cargo:rustc-env=GIT_COMMIT_MESSAGE={}", git_commit_message);
        println!("cargo:rustc-env=GIT_REPO={}", git_repo);

        // Set rerun if changed to .git/HEAD
        println!("cargo:rerun-if-changed=.git/HEAD");
    } else {
        println!("cargo:rustc-env=GIT_COMMIT_HASH=Unknown");
        println!("cargo:rustc-env=GIT_COMMIT_MESSAGE=Unknown");
        println!("cargo:rustc-env=GIT_REPO=Unknown");
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
                println!("cargo:rustc-env=CPU_MODEL={}", model);
                model_found = true;
                break;
            }
        }

        if !model_found {
            println!("cargo:rustc-env=CPU_MODEL=Unknown CPU");
        }

        println!("cargo:rerun-if-changed=/proc/cpuinfo");
    } else {
        println!("cargo:rustc-env=CPU_MODEL=Unknown CPU");
    }

    // rustc version
    let rustc_version = Command::new("rustc").args(["-V"]).output()?;

    let rustc_version = String::from_utf8(rustc_version.stdout)?.replace('\n', "");

    // Strip out extra data from rustc version by splitting by ( and taking the first part
    // E.g. rustc 1.79.0-nightly (ab5bda1aa 2024-04-08) becomes rustc 1.79.0-nightly
    let rustc_version = rustc_version.split('(').next().unwrap().to_string();

    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version);

    // Get profile
    let profile = std::env::var("PROFILE").unwrap_or("unknown".to_string());

    println!("cargo:rustc-env=CARGO_PROFILE={}", profile);

    Ok(())
}

fn main() -> Result<(), Error> {
    set_stats()?;

    // Run the autogen stuff
    autogen_modules_mod_rs()?;

    Ok(())
}
