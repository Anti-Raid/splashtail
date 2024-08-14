use crate::Error;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbed};
use splashcore_rs::jobserver::Task;

pub fn get_icon_of_state(state: &str) -> String {
    match state {
        "pending" => ":hourglass:",
        "running" => ":hourglass_flowing_sand:",
        "completed" => ":white_check_mark:",
        "failed" => ":x:",
        _ => ":question:",
    }
    .to_string()
}

pub fn embed<'a>(
    base_api_url: &str,
    task: &Task,
    pre_embeds: Vec<serenity::all::CreateEmbed<'a>>,
    show_status: bool,
) -> Result<poise::CreateReply<'a>, Error> {
    let mut task_statuses: Vec<String> = Vec::new();
    let mut task_statuses_length = 0;
    let mut components = Vec::new();

    let task_state = &task.state;

    if show_status {
        for status in &task.statuses {
            if task_statuses_length > 2500 {
                // Keep removing elements from start of array until we are under 2500 characters
                while task_statuses_length > 2500 {
                    let removed = task_statuses.remove(0);
                    task_statuses_length -= removed.len();
                }
            }

            let mut add = format!("`{}` {}", status.level, status.msg);

            let mut vs = Vec::new();

            let bdi = status.bot_display_ignore.clone().unwrap_or_default();

            for (k, v) in status.extra_info.iter() {
                if bdi.contains(k) {
                    continue;
                }

                vs.push(format!("{}={}", k, serde_json::to_string(v)?));
            }

            if !vs.is_empty() {
                add += &format!(" {}", vs.join(", "));
            }

            add = add.chars().take(500).collect::<String>()
                + if add.len() > 500 { "..." } else { "" };

            add += &format!(" | <t:{}:R>", status.ts.round());

            task_statuses_length += if add.len() > 500 { 500 } else { add.len() };
            task_statuses.push(add);
        }
    }

    let mut description = format!(
        "{} Task state: {}\nTask ID: {}\n\n{}",
        get_icon_of_state(task_state.as_str()),
        task_state,
        task.task_id,
        task_statuses.join("\n")
    );

    if task.state == "completed" {
        if let Some(ref output) = task.output {
            let furl = format!(
                "{}/tasks/{}/ioauth/download-link",
                base_api_url, task.task_id
            );
            description += &format!("\n\n:link: [Download {}]({})", output.filename, &furl);

            components.push(CreateActionRow::Buttons(vec![CreateButton::new_link(furl)
                .label("Download")
                .emoji('📥')]));
        }
    }

    let embed = CreateEmbed::default()
        .title("Task Status")
        .description(description)
        .color(serenity::all::Colour::DARK_GREEN);

    let mut msg = poise::CreateReply::default();

    for pre_embed in pre_embeds {
        msg = msg.embed(pre_embed);
    }

    msg = msg.embed(embed).components(components);

    Ok(msg)
}
