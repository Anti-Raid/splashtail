extern crate vergen;
use anyhow::Result;
use vergen::*;

/// Build src/modules/mod.rs based on the folder listing of src/modules
fn autogen_modules_mod_rs() -> Result<()> {
    const MODULE_TEMPLATE: &str = r#"
// Auto-generated by build.rs
{module_use_list}

/// List of modules available. Not all may be enabled
pub fn modules() -> Vec<crate::silverpelt::Module> {
    vec![
        {module_list}
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

    // Construct module_uses_list
    let mut module_use_list = Vec::new();

    for module in &module_list {
        module_use_list.push(format!("mod {};", module));
    }

    let module_use_list = module_use_list.join("\n");

    // Construct module_list
    let mut module_dat_list = Vec::new();

    for module in &module_list {
        module_dat_list.push(format!("{}::module(),", module));
    }

    let module_list = module_dat_list.join("\n        ");

    let module_list = MODULE_TEMPLATE
        .replace("{module_use_list}", &module_use_list)
        .replace("{module_list}", &module_list);

    let module_list = module_list.trim().to_string();

    std::fs::write("src/modules/mod.rs", module_list)?;

    Ok(())
}

fn main() -> Result<()> {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Normal;

    *config.git_mut().semver_kind_mut() = SemverKind::Normal;

    *config.git_mut().semver_mut() = true;

    *config.git_mut().semver_dirty_mut() = Some("-dirty");

    vergen(config)?;

    // Run the autogen stuff
    autogen_modules_mod_rs()?;

    Ok(())
}