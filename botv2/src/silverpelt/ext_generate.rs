/// We use a test to generate the .generated/serenity.json
#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn generate_serenity_perms_json() {
        println!(
            "Saving serenity_perms.json {}",
            std::env::current_dir().unwrap().display()
        );

        let perms: indexmap::IndexMap<String, u64> =
            serenity::model::permissions::Permissions::all()
                .iter()
                .map(|p| (p.to_string(), p.bits()))
                .collect();

        let perms_json = serde_json::to_string_pretty(&perms).unwrap();

        // Make .generated directory if it doesn't exist
        std::fs::create_dir_all(".generated").unwrap();

        let mut file = File::create(".generated/serenity_perms.json").unwrap();

        file.write_all(perms_json.as_bytes()).unwrap();
    }
}
