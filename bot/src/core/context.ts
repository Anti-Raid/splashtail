import { EmbedBuilder, InteractionResponse, Message, InteractionDeferReplyOptions, AutocompleteInteraction, ChatInputCommandInteraction, Guild, APIActionRowComponent, APIButtonComponent, APIChannelSelectComponent, APIMentionableSelectComponent, APIRoleSelectComponent, APIStringSelectComponent, APITextInputComponent, APIUserSelectComponent, APIMessageActionRowComponent, JSONEncodable, ActionRowData } from "discord.js";
import { AntiRaid } from "./client";

export type Component = any // for now

export interface ContextReply {
    content?: string;
    embeds?: EmbedBuilder[];
    ephemeral?: boolean;
    components?: Component[];
    fetchReply?: boolean;
}

export interface ContextEdit {
    content?: string;
    embeds?: EmbedBuilder[];
    components?: Component[];
}

/**
 * Contains the current state of the reply
 */
export enum ContextReplyStatus {
    /**
     * The reply has not been sent yet
     */
    Pending = "pending",
    /**
     * The reply has been sent
     */
    Replied = "replied",
    /**
     * The reply has been deferred
     */
    Deferred = "deferred",
}

export class CommandContext {
    client: AntiRaid;
    guild: Guild;
    interaction: ChatInputCommandInteraction
    private _replyState: ContextReplyStatus = ContextReplyStatus.Pending;

    constructor(client: AntiRaid, interaction: ChatInputCommandInteraction) {
        this.client = client;
        this.interaction = interaction;
        this.guild = interaction.guild;
    }

    get replyStatus() {
        return this._replyState;
    }

    public async reply(data: ContextReply): Promise<Message<boolean> | InteractionResponse<boolean>> {
        if(process.env.LOG_REPLYSTATE) {
            this.client.logger.debug("Context", "ReplyState", this._replyState, "Data", JSON.stringify(data))
        }
        
        if(this._replyState != ContextReplyStatus.Pending) {
            return await this.interaction.followUp(data)
        }

        if(data.fetchReply == undefined) {
            data.fetchReply = true
        }

        let res = await this.interaction.reply({
            content: data.content,
            embeds: data.embeds,
            ephemeral: data.ephemeral,
            components: data.components,
            fetchReply: data.fetchReply
        })

        this._replyState = ContextReplyStatus.Replied;

        return res
    }

    public async edit(data: ContextEdit) {
        await this.interaction.editReply({
            content: data.content,
            embeds: data.embeds,
            components: data.components,
        })
    }

    public async delete() {
        await this.interaction.deleteReply();
    }

    public async followUp(data: ContextReply) {
        if(this._replyState == ContextReplyStatus.Pending) {
            throw new Error("Cannot follow up before replying")
        }
        await this.interaction.followUp({
            content: data.content,
            embeds: data.embeds,
            ephemeral: data.ephemeral,
            components: data.components,
            fetchReply: data.fetchReply
        })
    }

    /**
     * Defers the reply to the interaction
     * @param options The options to defer the reply with
     */
    public async defer(options?: InteractionDeferReplyOptions) {
        if(this._replyState != ContextReplyStatus.Pending) {
            throw new Error("Cannot defer to an interaction that has already been responded to")
        }
        await this.interaction.deferReply(options);

        this._replyState = ContextReplyStatus.Deferred;
    }
}

export class AutocompleteContext {
    client: AntiRaid;
    interaction: AutocompleteInteraction

    constructor(client: AntiRaid, interaction: AutocompleteInteraction) {
        this.client = client;
        this.interaction = interaction;
    }
}