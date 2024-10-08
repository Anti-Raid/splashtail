mod tester;

use std::fs::File;
use std::io::Write;

fn generate_serenity_perms_json() {
    let mut perms: indexmap::IndexMap<String, u64> =
        serenity::model::permissions::Permissions::all()
            .iter()
            .map(|p| (p.to_string(), p.bits()))
            .collect();

    perms.sort_by(|_ka, va, _kb, vb| va.cmp(vb));

    let perms_json = serde_json::to_string_pretty(&perms).unwrap();

    let mut file = File::create("serenity_perms.json").unwrap();

    file.write_all(perms_json.as_bytes()).unwrap();
}

fn generate_channel_types_json() {
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

    let mut file = File::create("channel_types.json").unwrap();
    file.write_all(channel_types_json.as_bytes()).unwrap();

    let mut file = File::create("channel_types_inv.json").unwrap();
    file.write_all(channel_types_inv_json.as_bytes()).unwrap();
}

#[tokio::main]
async fn main() {
    println!(
        "Current dir: {}",
        std::env::current_dir().unwrap().display(),
    );

    // Get the first argument from cmd line
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("genassets") => {
            println!("Saving serenity_perms.json");

            generate_serenity_perms_json();

            println!("Saving channel_types.json/channel_types_inv.json");

            generate_channel_types_json();
        }
        Some("test") => {
            crate::tester::run_tester().await;
        }
        _ => {
            println!("No/unknown command specified. Use 'genassets' [generate build assets] or 'test' [test bot with some sanity checks]");
        }
    }
}
