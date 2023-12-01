import { Client, GatewayIntentBits, ActivityType, codeBlock, EmbedBuilder, Events, CommandInteraction, Message, InteractionResponse, Routes, Team, SlashCommandBuilder, Interaction, ModalSubmitInteraction, Colors, PermissionsBitField, TeamMember, AutocompleteInteraction } from "discord.js";
import { AutocompleteContext, CommandContext, ContextReply } from "./context";
import { Logger } from "./logger";
import { readFileSync, readdirSync } from "node:fs";
import { Config } from "./config";
import { createGuildIfNotExists } from "./common/guilds/guildBase";
import { parse } from "yaml";
import { validateAction } from "./common/poststats";
import { BotRedis } from "./redis";

export class FinalResponse {
    private dummyResponse: boolean // If true, then there is no final response (all processing is done in the command itself)
    private reply: ContextReply

    constructor() {}

    static dummy() {
        let response = new FinalResponse()
        response.dummyResponse = true
        return response
    }

    static reply(reply: ContextReply) {
        let response = new FinalResponse()
        response.reply = reply
        return response
    }

    /**
     * Mark the response as a dummy response or not (if true, then there is no final response (all processing is done in the command itself))
     * @param dummyResponse If true, then there is no final response (all processing is done in the command itself)
     */
    setDummyResponse(dummyResponse: boolean) {
        this.dummyResponse = dummyResponse
    }

    /**
     * Sets the final response
     * @param reply The final response
     */
    setFinalResponse(reply: ContextReply) {
        this.reply = reply
    }

    /**
     * 
     * @param ctx The context of the command
     * @returns The final response
     */
    async handle(ctx: CommandContext) {
        if(this.dummyResponse) {
            return
        }

        return await ctx.reply(this.reply)
    }
}

export enum BotStaffPerms {
    Owner,
}

export interface Command {
    userPerms: (PermissionsBitField | bigint)[];
    botPerms: (PermissionsBitField | bigint)[];
    botStaffPerms?: BotStaffPerms[];
    interactionData: SlashCommandBuilder | Omit<SlashCommandBuilder, "addSubcommand" | "addSubcommandGroup">;
    onLoad?: () => Promise<void>;
    execute: (context: CommandContext) => Promise<FinalResponse>;
    autocomplete?: (context: AutocompleteContext) => Promise<void>;
}

export class AntiRaid extends Client {
    commands: Map<string, Command> = new Map();
    logger: Logger;
    clusterId: number;
    shardCountAtStart: number;
    redis: BotRedis;
    private _config: Config;
    private hasLoadedListeners: boolean = false;
    private teamOwners: string[] = []
    allClustersLaunched: boolean = false

    constructor(clusterId: number, clusterName: string, shardIds: number[], shardCount: number) {
        super({
            shards: shardIds,
            shardCount: shardCount,
            intents: [
                GatewayIntentBits.Guilds,
                GatewayIntentBits.GuildMembers,
                GatewayIntentBits.MessageContent,
                GatewayIntentBits.GuildMessages,
            ],
        })

        this.clusterId = clusterId
        this.shardCountAtStart = shardCount
        this.logger = new Logger(`${clusterName} (${clusterId})`)
        this._config = this.loadConfig()
        this.redis = new BotRedis(this)
    }

    /**
     * Loads the config for the bot. This should never need to be called outside of the constructor
    */
    private loadConfig(): Config {
        this.logger.info("Config", "Loading config.yaml")
    
        let parsed = parse(readFileSync("../config.yaml").toString('utf-8'))
    
        if(!parsed?.discord_auth?.client_id) throw new Error("discord_auth.client_id is required in config.yaml")
        if(!parsed?.servers?.main) throw new Error("servers.main is required in config.yaml")
        if(!parsed.bot_lists) throw new Error("bot_lists is required in config.yaml")
    
        for (const botList of parsed.bot_lists) {
            if(!botList.name) throw new Error("name is required in bot_lists in config.yaml")
            if(!botList.api_url) throw new Error("api_url is required in bot_lists in config.yaml")
            if(!botList.api_token) throw new Error("api_token is required in bot_lists in config.yaml")
            if(!botList.auth_format) throw new Error("auth_format is required in bot_lists in config.yaml")
    
            if(botList.post_stats) {
                validateAction(botList.post_stats)
            }
        }
    
        this.logger.info("Config", "Loaded config.yaml")
    
        return parsed
    }

    /**
     * Returns the config of the bot
     */
    get config(): Config {
        return this._config
    }

    /**
     * Returns the owners of the bot
     */
    get botOwners(): string[] {
        return this.teamOwners
    }

    /**
     * Prepares the bot for use
     */
    async prepare() {
        if(!this.hasLoadedListeners) {
            this.loadEventListeners()
        }

        // Fetch team owners of bot
        this.rest.setToken(this.config.discord_auth.token)
        let data = await this.rest.get(Routes.oauth2CurrentApplication())
        this.logger.info("Discord", "Loaded application", data)

        // @ts-expect-error
        let teamMembers: TeamMember[] = data?.team?.members

        this.teamOwners = teamMembers.map(member => member.user.id)

        this.logger.info("Discord", `Loaded ${this.teamOwners.length} team owners`, this.teamOwners)

        await this.loadCommands()

        this.logger.info("Discord", `Loaded ${this.commands.size} commands`, this.commands)

        await this.redis.load()
    }

    /**
     * Starts the bot
     */
    async start() {
        await this.prepare()
        await this.login(this.config.discord_auth.token)
    }

    /**
     * This function waits until a button is pressed then returns the customId of the button
    */
    async waitForButtons(customIds: string[], timeout: number = 60000) {
        return new Promise<string>((resolve, reject) => {
            let listener = (interaction: Interaction) => {
                if(interaction.isButton()) {
                    if(customIds.includes(interaction.customId)) {
                        this.off(Events.InteractionCreate, listener)
                        resolve(interaction.customId)
                    }
                }
            }

            this.on(Events.InteractionCreate, listener)

            setTimeout(() => {
                this.off(Events.InteractionCreate, listener)
                reject("Timed out")
            }, timeout)
        })
    }

    /**
     * This function waits until a modal is submitted based on custom id then returns the customId of the modal and the response
     */
    async waitForModal(customId: string, timeout: number = 60000) {
        return new Promise<{ customId: string, response: ModalSubmitInteraction }>((resolve, reject) => {
            let listener = (interaction: Interaction) => {
                if(interaction.isModalSubmit()) {
                    if(interaction.customId == customId) {
                        this.off(Events.InteractionCreate, listener)
                        resolve({ customId: interaction.customId, response: interaction })
                    }
                }
            }

            this.on(Events.InteractionCreate, listener)

            setTimeout(() => {
                this.off(Events.InteractionCreate, listener)
                reject("Timed out")
            }, timeout)
        })
    }
    
    /**
     * This function is called when the bot is ready
     */
    private async onReady() {
        this.user.setActivity("Development of v6.0.0", {
            type: ActivityType.Watching,
        });
    
        this.user.setStatus("dnd");
    
        this.logger.success("Discord", `Connected as ${this.user.username}!`);

        await this.redis.launchNext()
    }

    /**
     * This function handles all the bot staff permissions
     * @param ctx The context of the command
     * @param perms The permissions to check
     * @returns true if the user has the staff perms needed, else false
     */
    private async handleBotStaffPerms(ctx: CommandContext | AutocompleteContext, perms: BotStaffPerms[]) {
        if(perms?.length == 0) return true

        if(perms?.includes(BotStaffPerms.Owner)) {
            if(!this.botOwners.includes(ctx.interaction.user.id)) {
                if(ctx instanceof CommandContext) {
                    await ctx.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setTitle("Bot Owners Only")
                                .setDescription(
                                    `This command can only be used by **owners** of the bot.`
                                )
                                .setColor(Colors.Red)
                        ]
                    });
                }
                return false;
            }
        }

        return true
    }

    /**
     * This function handles all the bot/user permissions
     * @param ctx The context of the command
     * @param command The command to check
     * @returns true if the user has the staff perms needed, else false
     */
    private async handlePermissions(ctx: CommandContext | AutocompleteContext, command: Command) {
        if(ctx.interaction.guild) {
            // Always ensure the guild is created on command use
            await createGuildIfNotExists(ctx.interaction.guild)
        }

        if(command.userPerms.length > 0) {
            if(!ctx.interaction.guild) {
                if(ctx instanceof CommandContext) {
                    await ctx.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setTitle("Guild Only")
                                .setDescription(
                                    `This command can only be used in a guild.`
                                )
                                .setColor(Colors.Red)
                        ]
                    });
                }
                return false;
            }

            if(!ctx.interaction.memberPermissions.has(command.userPerms)) {
                if(ctx instanceof CommandContext) {
                    try {
                        await ctx.reply({
                            embeds: [
                                new EmbedBuilder()
                                    .setTitle("Insufficient Permissions")
                                    .setDescription(
                                        `You do not have the required permissions to run this command. You need the following permissions: \`${command.userPerms.join(", ")}\``
                                    )
                                    .setColor(Colors.Red)
                            ]
                        });
                    } catch (err) {
                        this.logger.error(`Command (${command.interactionData.name})`, "Error when handling error:", err);
                    }
                }
                return false;
            }
        }

        if(command.botPerms.length > 0) {
            if(!ctx.interaction.appPermissions.has(command.botPerms)) {
                if(ctx instanceof CommandContext) {
                    await ctx.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setTitle("Insufficient Permissions")
                                .setDescription(
                                    `I do not have the required permissions to run this command. I need the following permissions: \`${command.botPerms.join(", ")}\``
                                )
                                .setColor(Colors.Red)
                        ]
                    });
                }
                return false;
            }
        }

        return true
    }

    /**
     * This is the core event listener for interactions
     */
    private async onInteraction(interaction: Interaction) {
        // Slash Command
        if (interaction.isChatInputCommand()) {
            let ctx = new CommandContext(this, interaction)

            const command = this.commands.get(interaction.commandName);
    
            if (!command) {
                try {
                    await ctx.reply({
                        embeds: [
                            new EmbedBuilder()
                                .setTitle("Command Unavailable")
                                .setDescription(
                                    `The command \`${interaction.commandName}\` is not available at this time`
                                )
                                .setColor(Colors.Red)
                        ]
                    });
                } catch (error) {
                    this.logger.error(`Command (${interaction.commandName})`, "Error when handling error:", error);
                }
                return;
            }

            let bsp = await this.handleBotStaffPerms(ctx, command.botStaffPerms)
            if(!bsp) {
                return
            }

            let pc = await this.handlePermissions(ctx, command)
            if(!pc) {
                return
            }
    
            try {
                let fr = await command.execute(ctx);

                if(fr) {
                    await fr.handle(ctx)
                }

                return
            } catch (error) {
                this.logger.error(`Command (${interaction.commandName})`, error);
                
                try {
                    await ctx.reply(
                    {
                            embeds: [
                                new EmbedBuilder()
                                    .setTitle("An Error Occurred")
                                    .setDescription(
                                        `An error occurred while executing the command \`${interaction.commandName}\`: ${error?.toString()?.slice(0, 2000)}`
                                    )
                                    .setColor(Colors.Red)
                            ]
                    }
                    );
                } catch (error) {
                    this.logger.error(`Command (${interaction.commandName})`, "Error when handling error:", error);
                }
                return;
            }
        }
    
        // Autocomplete
        if (interaction.isAutocomplete()) {
            const command = this.commands.get(
                interaction.commandName
            );
            if (!command) {
                return this.logger.error(
                    `Autocomplete (${interaction.commandName})`,
                    "Command does not exist"
                );
            }

            if (!command.autocomplete) {
                return this.logger.error(
                    `Autocomplete (${interaction.commandName})`,
                    "Command does not have an autocomplete function"
                );
            }
    
            let ctx = new AutocompleteContext(this, interaction)

            if(!this.handleBotStaffPerms(ctx, command.botStaffPerms)) {
                return
            }

            if(!this.handlePermissions(ctx, command)) {
                return
            }

            try {
                return await command.autocomplete(ctx);
            } catch (error) {
                return this.logger.error(
                    `Autocomplete (${interaction.commandName})`,
                    error
                );
            }
        }
    }

    /**
     * Loads all commands of the bot
     */
    private async loadCommands() {
        if(this.commands.size > 0) {
            this.logger.error("Discord", "Commands have already been loaded")
            return false
        }

        // Commands
        const commandFiles = readdirSync("build/commands")
            .filter((file) => file.endsWith(".js"));
        
        for (const file of commandFiles) {
            this.logger.info("Loader", `Loading command ${file.replace(".js", "")}`)
            const command: Command = (await import(`../commands/${file}`))?.default;

            if(!command) {
                throw new Error(`Invalid command ${file.replace(".js", "")}. Please ensure that you are exporting the command as default using \`export default command;\``)
            }

            let neededProps = ["execute", "interactionData"]

            for(let prop of neededProps) {
                if(!command[prop]) {
                    throw new Error(`Command ${file} is missing property ${prop}`)
                }
            }

            if(command.interactionData.name != file.replace(".js", "")) {
                throw new Error(`Command ${file} has an invalid name. Please ensure that the name of the command is the same as the file name`)
            }

            this.commands.set(command.interactionData.name, command);

            if(command.onLoad) {
                await command.onLoad()
            }
        }
    }

    /**
     * Loads all event listeners of the bot
     */
    private loadEventListeners() {
        if(this.hasLoadedListeners) {
            this.logger.error("Discord", "Event listeners have already been loaded")
            return false
        }

        this.once(Events.ClientReady, this.onReady);        

        // Discord Debug Event
        this.on(Events.Debug, (info) => this.logger.debug("Discord", info));

        // Discord Error Event
        this.on(Events.Error, (error) => this.logger.error("Discord", error));

        this.on(Events.ShardError, (error, id) => this.logger.error("Discord", `Shard ${id} error`, error));

        this.on(Events.InteractionCreate, async (interaction) => {
            try {
                await this.onInteraction(interaction).catch(err => {
                    this.logger.error("Discord", "Error when handling interaction", err)
                })
            } catch (err) {
                this.logger.error("Discord", "Error when handling interaction", err)
            }
        });  
                
        process.on("uncaughtException", (e) => {
            this.logger.error("Discord", "Uncaught Exception", e);
        });        

        this.hasLoadedListeners = true
    }
}
