type Error = Box<dyn std::error::Error + Send + Sync>;

fn _get_serenity_path() -> Result<String, Error> {
    // Find the commit of serenity used by the project
    let cargo_lock = std::fs::read_to_string("Cargo.lock")?;

    /* Skip to lines:
    [[package]]
    name = "serenity"
         */
    let serenity_start = cargo_lock
        .lines()
        .position(|x| x.contains("name = \"serenity\""));

    if serenity_start.is_none() {
        return Err("serenity not found in Cargo.lock".into());
    }

    let serenity_start = serenity_start.unwrap();

    // Now look for source
    let serenity_url = cargo_lock
        .lines()
        .skip(serenity_start)
        .position(|x| x.contains("source = \""));

    if serenity_url.is_none() {
        return Err("serenity source not found in Cargo.lock".into());
    }

    let serenity_url = serenity_url.unwrap();

    // Get the full line
    let serenity_url = cargo_lock
        .lines()
        .nth(serenity_start + serenity_url)
        .unwrap();

    let commit = serenity_url.split('#').last().unwrap();

    // From the long commit, get the short commit (8 chars)
    let commit = commit[0..7].to_string();

    //println!("cargo:warning=serenity commit: {}", commit);

    // First check serenity::model::event::FullEvent
    // Find serenity in ~/.cargo/git/checkouts
    let serenity_path = std::env::var("CARGO_HOME")?.to_string() + "/git/checkouts/serenity-*";

    let serenity_path = glob::glob(&serenity_path)?.collect::<Result<Vec<_>, _>>()?;

    // Emit a warning
    /* for path in serenity_path.iter() {
        println!("cargo:warning=serenity path: {:?}", path);
    } */

    let base_path = serenity_path[0].as_path().to_string_lossy().to_string() + "/" + &commit;

    //println!("cargo:warning=serenity base path: {}", base_path);

    Ok(base_path)
}

// EventStruct -> unflattened args
fn _get_serenity_events(
) -> Result<indexmap::IndexMap<String, indexmap::IndexMap<String, String>>, Error> {
    let base_path = _get_serenity_path()?;
    // Find enum FullEvent
    let models_file = std::fs::read_to_string(base_path + "/src/model/event.rs")?;
    let models_file = models_file.lines().collect::<Vec<&str>>();

    // Find a struct with Event in the name
    let mut events = indexmap::IndexMap::new();

    let mut current_event_marker: Option<String> = None;
    let mut current_event_fields: String = String::new();

    for line in models_file.iter() {
        if let Some(ref current_event) = current_event_marker {
            if line.contains("}") {
                let args = _unflatten_args(&current_event_fields);
                current_event_fields.clear();

                events.insert(current_event.clone(), args);

                current_event_marker = None;
            } else {
                if !line.contains("pub") {
                    continue;
                }

                current_event_fields.push_str(line.replace("pub", "").trim());
            }

            continue;
        }

        if line.contains("pub struct") && line.contains("Event") {
            // the struct will be of the form `pub struct EventName <any> {`
            //
            // Split whitespace, then find the element with Event in it
            let line = line.split_whitespace().collect::<Vec<&str>>();

            let event = line.iter().find(|x| x.contains("Event")).unwrap();

            events.insert(event.to_string(), indexmap::IndexMap::new());

            current_event_marker = Some(event.to_string());
        }
    }

    Ok(events)
}

#[derive(Debug, Clone)]
enum ExpandEventsCiEventCheck {
    /// Check that all fields of the event are satisfied by insert_fields or insert_optional_fields
    Event {
        var: String,
        event_struct: String,
        args: Vec<String>,
    },
    None {
        args: Vec<String>,
    },
}

impl ExpandEventsCiEventCheck {
    fn parse(s: &str) -> Self {
        let s = s.trim();

        let args_split = s.split("/").collect::<Vec<&str>>();
        let s = args_split[0].trim();
        let args = args_split[1..]
            .iter()
            .map(|x| x.trim().to_string())
            .collect::<Vec<String>>();

        // Simple case of none
        if s == "none" {
            return Self::None { args };
        }

        if s.starts_with("event:") {
            let s = s.replace("event:", "");

            let s = s.split(",").collect::<Vec<&str>>();

            let var = s[0].to_string();
            let event_struct = s[1].to_string();

            return Self::Event {
                var,
                event_struct,
                args,
            };
        }

        panic!("Invalid tagged ci event: {}", s);
    }
}

/// A parsed Ci event coming from expand_events
#[derive(Debug, Clone)]
struct ExpandEventsCurrentWorkingCiEvent {
    check: ExpandEventsCiEventCheck, // What to look for in the event
    variant_name: String,
    in_insert_field_call: (bool, i32), // Stores whether we are in an insert_field or insert_optional_field call
    insert_field_calls: Vec<String>,
}

fn ci_expand_events_parse() -> Result<Vec<ExpandEventsCurrentWorkingCiEvent>, Error> {
    let core_rs = std::fs::read_to_string("src/core.rs")?;
    let core_rs = core_rs.lines().collect::<Vec<&str>>();

    // Get all lines between // @ci.expand_event_check.start and // @ci.expand_event_check.end
    let mut lines = Vec::new();

    let mut found_start = false;
    let mut found_end = false;

    for line in core_rs.iter() {
        if line.contains("// @ci.expand_event_check.start") {
            found_start = true;
            continue;
        }

        if line.contains("// @ci.expand_event_check.end") {
            found_end = true;
            break;
        }

        if found_start && !found_end {
            lines.push(line);
        }
    }

    if !found_start || !found_end {
        return Err(
            "@ci.expand_event_check.start or @ci.expand_event_check.end comments missing".into(),
        );
    }

    if lines.is_empty() {
        return Err(
            "No lines found between @ci.expand_event_check.start and @ci.expand_event_check.end"
                .into(),
        );
    }

    // This stores the full list of all working ci events
    let mut working_ci_events: Vec<ExpandEventsCurrentWorkingCiEvent> = Vec::new();

    for line in lines {
        let line = line.trim();

        // Ensure no multi-line comments are present as this may break parsing
        if line.contains("/*") || line.contains("*/") {
            return Err(format!("Multi-line comment found in tagged ci event: {}. This is not allowed in expand_event block", line).into());
        }

        if !working_ci_events.is_empty() {
            let current_working_ci_event = working_ci_events.last_mut().unwrap();

            if line.contains("FullEvent::") {
                // Ignore non-none
                if line.contains("=> return None") {
                    continue;
                }

                // Format must be FullEvent::{variant_name} { .. } => {
                let line = line.replace("FullEvent::", "");
                let line = line.trim(); // Trim whitespace first to ensure split_whitespace works
                let line_split = line.split_whitespace().collect::<Vec<&str>>();

                let variant_name = line_split[0].to_string();

                if variant_name != current_working_ci_event.variant_name {
                    return Err(format!(
                        "Variant name mismatch: {} != {} [{}]",
                        variant_name, current_working_ci_event.variant_name, line
                    )
                    .into());
                }

                continue;
            }

            // If we see an insert_field or insert_optional_field call, set the flag
            if line.contains("insert_field") || line.contains("insert_optional_field") {
                if current_working_ci_event.in_insert_field_call.0 {
                    return Err(format!(
                        "Nested insert_field or insert_optional_field calls found in variant: {}",
                        current_working_ci_event.variant_name
                    )
                    .into());
                }

                if line.starts_with("//") {
                    return Err(format!(
                                    "Commented out insert_field or insert_optional_field call found in variant: {}",
                                    current_working_ci_event.variant_name
                                )
                                .into());
                }

                current_working_ci_event.in_insert_field_call.0 = true;
                current_working_ci_event.in_insert_field_call.1 += 1;
            }

            // If we are in an insert_field or insert_optional_field call, add it to map
            if current_working_ci_event.in_insert_field_call.0 {
                // Get value at position current_working_ci_event.in_insert_field_call.1 - 1 (1-indexed here due to += 1 above)
                if current_working_ci_event.insert_field_calls.len()
                    < current_working_ci_event.in_insert_field_call.1 as usize
                {
                    // Keep adding empty strings until we reach the required index
                    loop {
                        current_working_ci_event
                            .insert_field_calls
                            .push(String::new());

                        if current_working_ci_event.insert_field_calls.len()
                            == current_working_ci_event.in_insert_field_call.1 as usize
                        {
                            break;
                        }
                    }
                }
                let value = current_working_ci_event
                    .insert_field_calls
                    .get_mut((current_working_ci_event.in_insert_field_call.1 - 1) as usize)
                    .unwrap();

                // Append the line to the value
                let stripped_line = line.replace(['\n', '\t'], "");
                value.push_str(&stripped_line);
            }

            // If we see a semicolon, we are done with the insert_field or insert_optional_field call
            if line.contains(';') {
                current_working_ci_event.in_insert_field_call.0 = false;
            }
        } else {
            // If a non-None FullEvent is found without a tag, error out
            if line.contains("FullEvent::") {
                if line.contains("=> return None") {
                    continue;
                }
                return Err(
                    format!("FullEvent:: found without a tagged ci event: {}", line).into(),
                );
            }
        }

        // If a tagged ci event is found, parse it
        // Format: // @ci.expand_event_check VariantName CiEventCheck
        if line.starts_with("// @ci.expand_event_check") {
            let line = line.replace("// @ci.expand_event_check", "");
            let line = line.trim(); // Trim whitespace first to ensure split_whitespace works
            let line_split = line.split_whitespace().collect::<Vec<&str>>();

            let variant_name = line_split[0].to_string();
            let tag = ExpandEventsCiEventCheck::parse(line_split[1]);

            working_ci_events.push(ExpandEventsCurrentWorkingCiEvent {
                check: tag,
                variant_name,
                in_insert_field_call: (false, 0),
                insert_field_calls: Vec::new(),
            });
        }
    }

    //println!("cargo:warning=insert_field_calls: {:?}", working_ci_events);

    Ok(working_ci_events)
}

/// CI to check expand_events
fn ci_expand_events() -> Result<(), Error> {
    let serenity_event_struct_fields = _get_serenity_events()?;

    println!(
        "cargo:warning=serenity_event_struct_fields: {:?}",
        serenity_event_struct_fields
    );

    let working_ci_events = ci_expand_events_parse()?;

    let serenity_event_struct_fields = serenity_event_struct_fields
        .into_iter()
        .map(|(k, v)| (k, v.keys().map(|k| k.to_string()).collect::<Vec<String>>()))
        .collect::<indexmap::IndexMap<String, Vec<String>>>();

    for event in working_ci_events.iter() {
        match &event.check {
            ExpandEventsCiEventCheck::None { .. } => {}
            ExpandEventsCiEventCheck::Event {
                var, event_struct, ..
            } => {
                // An insert check is of the form insert_field/insert_optional_field(&mut fields, CATEGORY, VAR_STR, VAR_NAME)
                // We need to check that all VAR_NAME's starting with var[.] is present in the serenity_event_struct_fields
                let needed_fields: Vec<String> = serenity_event_struct_fields
                    .get(event_struct)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>();

                let mut missing_fields: Vec<String> = needed_fields.clone();

                for insert_field in event.insert_field_calls.iter() {
                    let insert_field = insert_field.trim();

                    if insert_field.is_empty() {
                        continue;
                    }

                    // Split by comma
                    let insert_field = insert_field.split(',').collect::<Vec<&str>>();

                    //println!("cargo:warning=var_name: {:?}", insert_field);

                    // From the last element, keep looping until we find an element starting with var[.]
                    let var_name = insert_field
                        .iter()
                        .rev()
                        .find(|x| x.trim().starts_with(&format!("{}.", var)));

                    let Some(var_name) = var_name else {
                        continue; // Skip this insert_field as it isn't what we're looking for
                    };

                    // Get rid of quotes
                    let var_name = var_name.replace(['"', '(', ')', ';'], "");

                    //println!("cargo:warning=var_name: {}", var_name);

                    // Split by dot and take the 2nd element (e.g. data.code -> code)
                    let var_name = var_name.split('.').collect::<Vec<&str>>()[1];
                    let var_name = var_name.to_string(); // Clone to remove borrow checker error

                    // Check if var_name is in missing_fields
                    if missing_fields.contains(&var_name) {
                        // Remove the field from missing_fields
                        let index = missing_fields.iter().position(|x| *x == var_name).unwrap();
                        missing_fields.remove(index);
                    } else {
                        return Err(format!(
                            "Field {} not found in event struct {}",
                            var_name, event_struct
                        )
                        .into());
                    }
                }

                if !missing_fields.is_empty() {
                    return Err(format!(
                        "Fields missing in event struct {}: {:?} (of {:?}) {:?}",
                        event_struct, missing_fields, needed_fields, event.insert_field_calls
                    )
                    .into());
                }
            }
        }
    }

    Ok(())
}

fn _unflatten_args(fields: &str) -> indexmap::IndexMap<String, String> {
    // Store the fields in the event_fields map
    let mut event_field_map: indexmap::IndexMap<String, String> = indexmap::IndexMap::new();

    let mut in_key: bool = true;
    let mut in_generic: usize = 0;

    // Parse fields character by character
    // Ex. guild_id: GuildId, current_state: ExtractMap<EmojiId, Emoji>
    // should become guild_id -> GuildId, current_state -> ExtractMap<EmojiId, Emoji>
    let mut key = String::new();
    let mut value = String::new();
    for c in fields.chars() {
        //println!("cargo:warning=c: {:?}, in_key: {:?}", c, in_key);
        if c == ':' {
            in_key = false;
            continue;
        } else if c == '<' {
            in_generic += 1;
        } else if c == '>' {
            in_generic -= 1;
        }

        if c == ',' && in_generic == 0 {
            in_key = true;

            // Insert the key-value pair into the map
            event_field_map.insert(key, value);

            // Reset key and value
            key = String::new();
            value = String::new();

            continue;
        }

        if in_key {
            if c == ' ' {
                continue;
            }

            key.push(c);
        } else {
            if c == ' ' && in_generic == 0 {
                continue;
            }

            value.push(c);
        }
    }

    // Insert the last key-value pair
    if !key.is_empty() {
        event_field_map.insert(key, value);
    }

    event_field_map
}

// CI to create template documentation
fn create_template_docs() -> Result<(), Error> {
    let serenity_path = _get_serenity_path()?;

    // Read src/client/event_handler.rs
    let event_handler = std::fs::read_to_string(serenity_path + "/src/client/event_handler.rs")?;

    // Remove out all lines preceding event_handler! macro
    let event_handler = event_handler.lines().collect::<Vec<&str>>();

    let mut event_handler = event_handler
        .iter()
        .skip_while(|x| !x.contains("event_handler!"))
        .collect::<Vec<&&str>>();

    if event_handler.is_empty() {
        return Err("event_handler! macro not found in event_handler.rs".into());
    }

    // Next, store all the fields of an event in an indexmap of [EventName] -> HashMap<Field, Type>
    //
    // A field is of the form: <EventName> { field_name: Type } ...
    let mut event_fields: indexmap::IndexMap<String, indexmap::IndexMap<String, String>> =
        indexmap::IndexMap::new();

    // Find all events
    for line in event_handler.iter() {
        let line = line.trim();
        //println!("cargo:warning=event_handler: {:?}", line);
        if line.contains("=>") {
            let line_split = line.split("=>").collect::<Vec<&str>>();
            let line = line_split[0].trim();

            // Get the event name
            let event_name = line.split_whitespace().collect::<Vec<&str>>()[0].to_string();

            // Get the fields
            let fields = line.split('{').collect::<Vec<&str>>()[1];
            let fields = fields.split('}').collect::<Vec<&str>>()[0];
            let fields = fields.trim();

            //println!("cargo:warning=fields: {:?}", fields);
            let event_field_map = _unflatten_args(fields);

            event_fields.insert(event_name, event_field_map);
        }
    }

    // Special case: if our event matches ExpandEventsCiEventCheck::Event, we need to add the fields to the event_fields map and remove the raw event struct
    let working_ci_events = ci_expand_events_parse()?;
    let serenity_event_struct_fields = _get_serenity_events()?;
    for working_ci_event in working_ci_events.iter() {
        match &working_ci_event.check {
            ExpandEventsCiEventCheck::None { .. } => {}
            ExpandEventsCiEventCheck::Event {
                var,
                event_struct,
                args,
            } => {
                println!("cargo:warning=var: {:?}", var);
                println!("cargo:warning=event_struct: {:?}", event_struct);
                println!(
                    "cargo:warning=event_struct: {:?}",
                    working_ci_event.variant_name
                );
                // Add the fields to the event_fields map
                let fields = event_fields
                    .get_mut(&working_ci_event.variant_name)
                    .unwrap();

                // Get the fields from serenity_event_struct_fields
                let serenity_event_struct_fields = serenity_event_struct_fields
                    .get(event_struct)
                    .unwrap()
                    .clone();

                // Remove the `var`
                fields.shift_remove(var).unwrap();

                // Insert the serenity_event_struct_fields
                for (k, v) in serenity_event_struct_fields.iter() {
                    fields.insert(k.clone(), v.clone());
                }
            }
        }
    }

    println!("cargo:warning=event_fields: {:?}", event_fields);

    #[derive(serde::Serialize)]
    pub struct EventFieldLocator {
        pub field: String,
        pub serenity_filename: String,
    }

    // Stores the event fields for each event, this is used in the website + for documenting the fields that events have
    let mut event_fields: indexmap::IndexMap<String, Vec<EventFieldLocator>> =
        indexmap::IndexMap::new();

    Ok(())
}

fn main() -> Result<(), Error> {
    ci_expand_events()?;
    create_template_docs()?;

    Ok(())
}
