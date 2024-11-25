use crate::Job;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbed};
use splashcore_rs::Error;

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
    job: &Job,
    pre_embeds: Vec<serenity::all::CreateEmbed<'a>>,
    show_status: bool,
) -> Result<poise::CreateReply<'a>, Error> {
    let mut job_statuses: Vec<String> = Vec::new();
    let mut job_statuses_length = 0;
    let mut components = Vec::new();

    let job_state = &job.state;

    if show_status {
        for status in &job.statuses {
            if job_statuses_length > 2500 {
                // Keep removing elements from start of array until we are under 2500 characters
                while job_statuses_length > 2500 {
                    let removed = job_statuses.remove(0);
                    job_statuses_length -= removed.len();
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

            job_statuses_length += if add.len() > 500 { 500 } else { add.len() };
            job_statuses.push(add);
        }
    }

    let mut description = format!(
        "{} Job state: {}\nJob ID: {}\n\n{}",
        get_icon_of_state(job_state.as_str()),
        job_state,
        job.id,
        job_statuses.join("\n")
    );

    if job.state == "completed" {
        if let Some(ref output) = job.output {
            let furl = format!("{}/jobs/{}/ioauth/download-link", base_api_url, job.id);
            description += &format!("\n\n:link: [Download {}]({})", output.filename, &furl);

            components.push(CreateActionRow::Buttons(
                vec![CreateButton::new_link(furl).label("Download").emoji('📥')].into(),
            ));
        }
    }

    let embed = CreateEmbed::default()
        .title("Status")
        .description(description)
        .color(serenity::all::Colour::DARK_GREEN);

    let mut msg = poise::CreateReply::default();

    for pre_embed in pre_embeds {
        msg = msg.embed(pre_embed);
    }

    msg = msg.embed(embed).components(components);

    Ok(msg)
}
