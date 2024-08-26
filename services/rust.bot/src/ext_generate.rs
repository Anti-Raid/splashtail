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

        let mut perms: indexmap::IndexMap<String, u64> =
            serenity::model::permissions::Permissions::all()
                .iter()
                .map(|p| (p.to_string(), p.bits()))
                .collect();

        perms.sort_by(|_ka, va, _kb, vb| va.cmp(vb));

        let perms_json = serde_json::to_string_pretty(&perms).unwrap();

        // Make .generated directory if it doesn't exist
        std::fs::create_dir_all(".generated").unwrap();

        let mut file = File::create(".generated/serenity_perms.json").unwrap();

        file.write_all(perms_json.as_bytes()).unwrap();
    }

    #[test]
    fn generate_channel_types_json() {
        println!(
            "Saving channel_types.json/channel_types_inv.json {}",
            std::env::current_dir().unwrap().display()
        );

        let mut channel_types = indexmap::IndexMap::<String, u8>::new();
        let mut channel_types_inv = indexmap::IndexMap::<u8, String>::new();

        // Keep looping until we hit an Unknown ChannelType
        let mut i: u8 = 0;

        loop {
            if i == u8::MAX {
                break;
            }

            // Hacky workaround for serenity ChannelType
            let i_serde = serde_json::to_value(i).unwrap();
            let channel_type: serenity::model::channel::ChannelType =
                serde_json::from_value(i_serde).unwrap();

            if channel_type.name().to_lowercase() != "unknown" {
                channel_types.insert(channel_type.name().to_string(), i);
                channel_types_inv.insert(i, channel_type.name().to_string());
            }

            i += 1
        }

        let channel_types_json = serde_json::to_string_pretty(&channel_types).unwrap();
        let channel_types_inv_json = serde_json::to_string_pretty(&channel_types_inv).unwrap();

        // Make .generated directory if it doesn't exist
        std::fs::create_dir_all(".generated").unwrap();

        let mut file = File::create(".generated/channel_types.json").unwrap();
        file.write_all(channel_types_json.as_bytes()).unwrap();

        let mut file = File::create(".generated/channel_types_inv.json").unwrap();
        file.write_all(channel_types_inv_json.as_bytes()).unwrap();
    }
}
