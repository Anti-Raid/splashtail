import { PlatformUser } from "./eureka-dovewing"

export interface AuthorizeRequest {
  code: string;
  redirect_uri: string;
  protocol: string;
  scope: string;
}
export interface UserSession {
  id: string;
  name?: string;
  user_id: string;
  created_at: string /* RFC3339 */;
  type: string;
  expiry: string /* RFC3339 */;
}
export interface CreateUserSession {
  name: string;
  type: string;
  expiry: number;
}
export interface CreateUserSessionResponse {
  user_id: string;
  token: string;
  session_id: string;
  expiry: string /* RFC3339 */;
}
export interface UserSessionList {
  sessions: (UserSession | undefined)[];
}
export interface TestAuth {
  auth_type: string;
  target_id: string;
  token: string;
}

/**
 * API configuration data
 */
export interface ApiConfig {
  main_server: string;
  support_server_invite: string;
  client_id: string;
}
/**
 * This represents a API Error
 */
export interface ApiError {
  context?: { [key: string]: string};
  message: string;
}
export interface BotState {
  commands: ApplicationCommand[];
}
export interface DashboardGuild {
  id: string;
  name: string;
  avatar: string;
  permissions: number /* int64 */;
}
export interface DashboardGuildData {
  guilds: (DashboardGuild | undefined)[];
  has_bot: string[];
  unknown_guilds: string[];
}
/**
 * Represents a user on Antiraid
 */
export interface User {
  user?: PlatformUser /* from eureka-dovewing.ts */;
  state: string;
  vote_banned: boolean;
  created_at: string /* RFC3339 */;
  updated_at: string /* RFC3339 */;
}
export interface UserGuildBaseData {
  owner_id: string;
  name: string;
  icon?: string;
  roles: SerenityRole[];
  user_roles: string[];
  bot_roles: string[];
  channels: GuildChannelWithPermissions[];
}
/**
 * SettingsExecute allows execution of a settings operation
 */
export interface SettingsExecute {
  operation: string;
  setting: string;
  fields: { [key: string]: any};
}
export interface DispatchResult {
  type: string;
  data: any;
}
/**
 * Sent on List Template Shop Templates
 */
export interface TemplateShopPartialTemplate {
  id: string;
  name: string;
  version: string;
  description: string;
  owner_guild: string;
  created_at: string /* RFC3339 */;
  last_updated_at: string /* RFC3339 */;
  friendly_name: string;
  events: string[];
  language: string;
  allowed_caps: string[];
  tags: string[];
}
/**
 * TemplateShopTemplate is the full template data sent on Get Template Shop Template
 * It includes the content of the template, which is not included in the partial template
 */
export interface TemplateShopTemplate {
  id: string;
  name: string;
  version: string;
  description: string;
  owner_guild: string;
  created_at: string /* RFC3339 */;
  last_updated_at: string /* RFC3339 */;
  friendly_name: string;
  events: string[];
  language: string;
  allowed_caps: string[];
  content: { [key: string]: string};
  tags: string[];
}

//////////
// source: ext_types.go

export type Permissions = string;
export interface SerenityRoleTags {
  bot_id?: string;
  integration_id?: string;
  premium_subscriber: boolean;
  subscription_listing_id?: string;
  available_for_purchase: boolean;
  guild_connections: boolean;
}
export interface SerenityRole {
  id: string;
  guild_id: string;
  color: number /* int */;
  name: string;
  permissions?: Permissions;
  position: number /* int16 */;
  tags?: SerenityRoleTags;
  icon?: string;
  unicode_emoji: string;
}
export interface GuildChannelWithPermissions {
  user: Permissions;
  bot: Permissions;
  channel?: Channel;
}

//////////
// source: stats.go

export interface ShardConn {
  status: string;
  real_latency: number /* int64 */;
  guilds: number /* int64 */;
  uptime: number /* int64 */;
  total_uptime: number /* int64 */;
}
export interface GetStatusResponse {
  resp: StatusEndpointResponse;
  shard_conns: { [key: number /* int64 */]: ShardConn};
  total_guilds: number /* int64 */;
}
export interface StatusEndpointResponse {
  uptime: number /* int64 */;
  managers: StatusEndpointManager[];
}
export interface StatusEndpointManager {
  display_name: string;
  shard_groups: ShardGroup[];
}
export interface ShardGroup {
  shards: number /* int64 */[][];
}
export interface Resp {
  ok: boolean;
  data?: StatusEndpointResponse;
}

// Discordgo related types
/**
 * A Channel holds all data related to an individual Discord channel.
 */
export interface Channel {
  /**
   * The ID of the channel.
   */
  id: string;
  /**
   * The ID of the guild to which the channel belongs, if it is in a guild.
   * Else, this ID is empty (e.g. DM channels).
   */
  guild_id: string;
  /**
   * The name of the channel.
   */
  name: string;
  /**
   * The topic of the channel.
   */
  topic: string;
  /**
   * The type of the channel.
   */
  type: ChannelType;
  /**
   * The ID of the last message sent in the channel. This is not
   * guaranteed to be an ID of a valid message.
   */
  last_message_id: string;
  /**
   * The timestamp of the last pinned message in the channel.
   * nil if the channel has no pinned messages.
   */
  last_pin_timestamp?: string /* RFC3339 */;
  /**
   * An approximate count of messages in a thread, stops counting at 50
   */
  message_count: number /* int */;
  /**
   * An approximate count of users in a thread, stops counting at 50
   */
  member_count: number /* int */;
  /**
   * Whether the channel is marked as NSFW.
   */
  nsfw: boolean;
  /**
   * Icon of the group DM channel.
   */
  icon: string;
  /**
   * The position of the channel, used for sorting in client.
   */
  position: number /* int */;
  /**
   * The bitrate of the channel, if it is a voice channel.
   */
  bitrate: number /* int */;
  /**
   * The recipients of the channel. This is only populated in DM channels.
   */
  recipients: (User | undefined)[];
  /**
   * A list of permission overwrites present for the channel.
   */
  permission_overwrites?: PermissionOverwrite[];
  /**
   * The user limit of the voice channel.
   */
  user_limit: number /* int */;
  /**
   * The ID of the parent channel, if the channel is under a category. For threads - id of the channel thread was created in.
   */
  parent_id: string;
  /**
   * Amount of seconds a user has to wait before sending another message or creating another thread (0-21600)
   * bots, as well as users with the permission manage_messages or manage_channel, are unaffected
   */
  rate_limit_per_user: number /* int */;
  /**
   * ID of the creator of the group DM or thread
   */
  owner_id: string;
  /**
   * ApplicationID of the DM creator Zeroed if guild channel or not a bot user
   */
  application_id: string;

  /**
   * Channel flags.
   */
  flags: ChannelFlags;
  /**
   * The IDs of the set of tags that have been applied to a thread in a forum channel.
   */
  applied_tags: string[];
}

/**
 * PermissionOverwriteType represents the type of resource on which
 * a permission overwrite acts.
 */
export type PermissionOverwriteType = number /* int */;
/**
 * The possible permission overwrite types.
 */
export const PermissionOverwriteTypeRole: PermissionOverwriteType = 0;
/**
 * The possible permission overwrite types.
 */
export const PermissionOverwriteTypeMember: PermissionOverwriteType = 1;
/**
 * A PermissionOverwrite holds permission overwrite data for a Channel
 */
export interface PermissionOverwrite {
  id: string;
  type: PermissionOverwriteType;
  deny: number /* int64 */;
  allow: number /* int64 */;
}

/**
 * ChannelType is the type of a Channel
 */
export type ChannelType = number /* int */;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildText: ChannelType = 0;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeDM: ChannelType = 1;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildVoice: ChannelType = 2;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGroupDM: ChannelType = 3;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildCategory: ChannelType = 4;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildNews: ChannelType = 5;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildStore: ChannelType = 6;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildNewsThread: ChannelType = 10;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildPublicThread: ChannelType = 11;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildPrivateThread: ChannelType = 12;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildStageVoice: ChannelType = 13;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildDirectory: ChannelType = 14;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildForum: ChannelType = 15;
/**
 * Block contains known ChannelType values
 */
export const ChannelTypeGuildMedia: ChannelType = 16;
/**
 * ChannelFlags represent flags of a channel/thread.
 */
export type ChannelFlags = number /* int */;
/**
 * ChannelFlagPinned indicates whether the thread is pinned in the forum channel.
 * NOTE: forum threads only.
 */
export const ChannelFlagPinned: ChannelFlags = 1 << 1;
/**
 * ChannelFlagRequireTag indicates whether a tag is required to be specified when creating a thread.
 * NOTE: forum channels only.
 */
export const ChannelFlagRequireTag: ChannelFlags = 1 << 4;


// App command stuff


/**
 * ApplicationCommand represents an application's slash command.
 */
export interface ApplicationCommand {
  id?: string;
  application_id?: string;
  guild_id?: string;
  version?: string;
  type?: ApplicationCommandType;
  name: string;
  name_localizations?: { [key: string]: string};
  /**
   * NOTE: DefaultPermission will be soon deprecated. Use DefaultMemberPermissions and Contexts instead.
   */
  default_permission?: boolean;
  default_member_permissions?: number /* int64 */;
  nsfw?: boolean;
  /**
   * Deprecated: use Contexts instead.
   */
  dm_permission?: boolean;
  contexts?: InteractionContextType[];
  integration_types?: ApplicationIntegrationType[];
  description?: string;
  description_localizations?: { [key: string]: string};
  options: (ApplicationCommandOption | undefined)[];
}

/**
 * ApplicationCommandType represents the type of application command.
 */
export type ApplicationCommandType = number /* uint8 */;
/**
 * ChatApplicationCommand is default command type. They are slash commands (i.e. called directly from the chat).
 */
export const ChatApplicationCommand: ApplicationCommandType = 1;
/**
 * UserApplicationCommand adds command to user context menu.
 */
export const UserApplicationCommand: ApplicationCommandType = 2;
/**
 * MessageApplicationCommand adds command to message context menu.
 */
export const MessageApplicationCommand: ApplicationCommandType = 3;

/**
 * InteractionContextType represents the context in which interaction can be used or was triggered from.
 */
export type InteractionContextType = number /* uint */;
/**
 * InteractionContextGuild indicates that interaction can be used within guilds.
 */
export const InteractionContextGuild: InteractionContextType = 0;
/**
 * InteractionContextBotDM indicates that interaction can be used within DMs with the bot.
 */
export const InteractionContextBotDM: InteractionContextType = 1;
/**
 * InteractionContextPrivateChannel indicates that interaction can be used within group DMs and DMs with other users.
 */
export const InteractionContextPrivateChannel: InteractionContextType = 2;

/**
 * ApplicationIntegrationType dictates where application can be installed and its available interaction contexts.
 */
export type ApplicationIntegrationType = number /* uint */;
/**
 * ApplicationIntegrationGuildInstall indicates that app is installable to guilds.
 */
export const ApplicationIntegrationGuildInstall: ApplicationIntegrationType = 0;
/**
 * ApplicationIntegrationUserInstall indicates that app is installable to users.
 */
export const ApplicationIntegrationUserInstall: ApplicationIntegrationType = 1;

/**
 * ApplicationCommandOptionType indicates the type of a slash command's option.
 */
export type ApplicationCommandOptionType = number /* uint8 */;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionSubCommand: ApplicationCommandOptionType = 1;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionSubCommandGroup: ApplicationCommandOptionType = 2;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionString: ApplicationCommandOptionType = 3;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionInteger: ApplicationCommandOptionType = 4;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionBoolean: ApplicationCommandOptionType = 5;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionUser: ApplicationCommandOptionType = 6;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionChannel: ApplicationCommandOptionType = 7;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionRole: ApplicationCommandOptionType = 8;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionMentionable: ApplicationCommandOptionType = 9;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionNumber: ApplicationCommandOptionType = 10;
/**
 * Application command option types.
 */
export const ApplicationCommandOptionAttachment: ApplicationCommandOptionType = 11;
/**
 * ApplicationCommandOption represents an option/subcommand/subcommands group.
 */
export interface ApplicationCommandOption {
  type: ApplicationCommandOptionType;
  name: string;
  name_localizations?: { [key: string]: string};
  description?: string;
  description_localizations?: { [key: string]: string};
  channel_types: ChannelType[];
  required: boolean;
  options: (ApplicationCommandOption | undefined)[];
  /**
   * NOTE: mutually exclusive with Choices.
   */
  autocomplete: boolean;
  choices: (ApplicationCommandOptionChoice | undefined)[];
  /**
   * Minimal value of number/integer option.
   */
  min_value?: number /* float64 */;
  /**
   * Maximum value of number/integer option.
   */
  max_value?: number /* float64 */;
  /**
   * Minimum length of string option.
   */
  min_length?: number /* int */;
  /**
   * Maximum length of string option.
   */
  max_length?: number /* int */;
}

/**
 * ApplicationCommandOptionChoice represents a slash command option choice.
 */
export interface ApplicationCommandOptionChoice {
  name: string;
  name_localizations?: { [key: string]: string};
  value: any;
}
