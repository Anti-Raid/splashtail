import { ActionRowBuilder, ButtonBuilder, ButtonStyle, EmbedBuilder } from "discord.js";
import { CommandContext, ContextEdit, Component } from "../context";
import { Task } from "../../generatedTypes/types";
import { AntiRaid } from "../client";
import sql from "../db";

export const getTask = async (taskId: string): Promise<Task> => {
    let data = await sql`SELECT allow_unauthenticated, task_name, output, task_info, statuses, task_for, expiry, state, created_at FROM tasks WHERE task_id = ${taskId}`

    if(data.count == 0) return null

    let task = data[0] as Task

    task.task_id = taskId

    return task
}

export interface PollTaskOptions {
    callback: (task: Task) => Promise<void>,
    timeout?: number,
    pollInterval?: number
}

export const pollTask = async (taskId: string, opts: PollTaskOptions): Promise<Task> => {
    if(!taskId) throw new Error("taskId is required")

    if(!opts?.pollInterval) {
        opts.pollInterval = 1000
    }

    let done = false

    if(opts?.timeout) {
        setTimeout(() => {
            done = true
        }, opts?.timeout)
    }

    let prevTask: Task = null
    while(!done) {
        let task = await getTask(taskId)

        if(!task) {
            throw new Error("Task not found")
        }

        if(prevTask) {
            // Prevent spamming of edits
            if(task?.state === prevTask?.state && JSON.stringify(task) === JSON.stringify(prevTask)) {
                await new Promise((resolve) => setTimeout(resolve, opts.pollInterval))
                continue
            }
        }

        prevTask = task

        await opts.callback(task)

        if(task.state != "pending" && task.state != "running") {
            return task
        }

        await new Promise((resolve) => setTimeout(resolve, opts.pollInterval))
    }
}

export const createTaskEmbed = (ctx: CommandContext, task: Task): ContextEdit => {
    let taskStatuses: string[] = []
    let taskStatusesLength = 0

    let taskState = task?.state

    for(let status of task.statuses) {
        if(taskStatusesLength > 2500) {
            // Keep removing elements from start of array until we are under 2500 characters
            while(taskStatusesLength > 2500) {
                let removed = taskStatuses.shift()
                taskStatusesLength -= removed.length
            }
        }

        let add = `\`${status?.level}\` ${status?.msg}`

        let vs: string[] = []
        for(let [k, v] of Object.entries(status || {})) {
            if(k == "level" || k == "msg" || k == "ts" || k == "botDisplayIgnore") continue
            if(status["botDisplayIgnore"]?.includes(k)) continue

            vs.push(`${k}=${typeof v == "object" ? JSON.stringify(v) : v}`)
        }

        if(vs.length > 0) add += ` ${vs.join(", ")}`

        add = add.slice(0, 500) + (add.length > 500 ? "..." : "")

        add += ` | \`[${new Date(status?.ts * 1000)}]\``

        taskStatuses.push(add)
        taskStatusesLength += (add.length > 500 ? 500 : add.length)
    }

    let emoji = ":white_check_mark:"

    switch (taskState) {
        case "pending":
            emoji = ":hourglass:"
            break;
        case "running":
            emoji = ":hourglass_flowing_sand:"
            break;
        case "completed":
            emoji = ":white_check_mark:"
            break;
        case "failed":
            emoji = ":x:"
            break;
    }

    let description = `${emoji} Task state: ${taskState}\nTask ID: ${task?.task_id}\n\n${taskStatuses.join("\n")}`
    let components: Component[] = []

    if(taskState == "completed") {
        if(task?.output?.filename) {
            description += `\n\n:link: [Download](${ctx.client.apiUrl}/tasks/${task?.task_id}/ioauth/download-link)`

            components.push(
                new ActionRowBuilder()
                .addComponents(
                    new ButtonBuilder()
                    .setLabel("Download")
                    .setStyle(ButtonStyle.Link)
                    .setURL(`${ctx.client.apiUrl}/tasks/${task?.task_id}/ioauth/download-link`)
                    .setEmoji("📥")
                )
                .toJSON()
            )    
        }
    }

    let embed = new EmbedBuilder()
    .setTitle("Creating backup")
    .setDescription(description)
    .setColor("Green")

    if(taskState == "completed") {
        embed.setFooter({
            text: "Backup created successfully"
        })
    }

    return {
        embeds: [embed],
        components
    }
}