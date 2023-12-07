import { ActionRowBuilder, ButtonBuilder, ButtonStyle, EmbedBuilder } from "discord.js";
import { Task } from "../coreTypes/tasks";
import { CommandContext, ContextEdit, Component } from "../context";

export const createTaskEmbed = (ctx: CommandContext, task: Task): ContextEdit => {
    let taskStatuses: string[] = []
    let taskStatusesLength = 0

    for(let status of task.statuses) {
        if(taskStatusesLength > 2500) {
            // Keep removing elements from start of array until we are under 2500 characters
            while(taskStatusesLength > 2500) {
                let removed = taskStatuses.shift()
                taskStatusesLength -= removed.length
            }
        }

        let add = `\`${status?.level}\` ${status?.msg}`
        taskStatuses.push(add)
        taskStatusesLength += add.length
    }

    let description = `:white_check_mark: Task state: ${task?.state}\n\n${taskStatuses.join("\n")}`
    let components: Component[] = []

    if(task?.state == "completed") {
        if(task?.output?.path) {
            description += `\n\n:link: [Download backup](${ctx.client.apiUrl}/tasks/${task?.task_id}/uauth/download`
        }

        components.push(
            new ActionRowBuilder(
                new ButtonBuilder()
                .setLabel("Download Backup")
                .setStyle(ButtonStyle.Link)
                .setURL(`${ctx.client.apiUrl}/ioauth/tasks/${task?.task_id}/download`)
                .setEmoji("📥")
                .toJSON()
            ).toJSON()
        )
    }

    let embed = new EmbedBuilder()
    .setTitle("Creating backup")
    .setDescription(description)
    .setColor("Green")

    if(task?.state == "completed") {
        embed.setFooter({
            text: "Backup created successfully"
        })
    }

    return {
        embeds: [embed],
        components
    }
}