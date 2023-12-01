export interface BotListAction {
    enabled: boolean,
    method: string,
    interval: number,
    url_format: string // Must be u#{url}?[key1]={key2} (brackets means that anything can be substituted in)
    data_format?: { [key: string]: string }
}

export interface BotList {
    name: string,
    api_url: string,
    api_token: string,
    auth_format: string, // Can be one of h#[header]/{token} or u#[token]={token} or b#[key]={token} (brackets means that anything can be substituted in)
    post_stats?: BotListAction
}

export interface Servers {
    main: string
}

export interface DiscordAuth {
    client_id: string,
    client_secret: string,
    token: string
}

export interface Config {
    discord_auth: DiscordAuth,
    servers: Servers,
    bot_lists: BotList[]
}