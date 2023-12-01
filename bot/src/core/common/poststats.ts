import { AntiRaid } from "../client"
import { BotList, BotListAction } from "../config"
import { getServerCount, getShardCount } from "./counts"

export function validateAction(action: BotListAction) {
    if(!action.enabled) throw new Error("<action>.enabled is required in bot_lists in config.yaml")
    if(!action.method) throw new Error("<action>.method is required in bot_lists in config.yaml")
    if(!action.interval) throw new Error("<action>.interval is required in bot_lists in config.yaml")
    if(!action.url_format) throw new Error("<action>.url_format is required in bot_lists in config.yaml")
}

function parseHeader(header: string, variables: { [key: string]: any }) {
    let parsed = header;
    for (const key in variables) {
        parsed = parsed.replaceAll(`{${key}}`, variables[key]);
    }
    return parsed;
}

function createData(botList: BotList, action: BotListAction, initialVariables: { [key: string]: any }) {
    let [mod, format, ...ext] = action.url_format.split("#");

    if(mod != "u") {
        throw new Error("Only u# is supported for url_format in bot_lists in config.yaml")
    }

    let variables = {
        ...initialVariables,
        url: botList.api_url,
        token: botList.api_token,
    }

    let url = parseHeader(format, variables)

    let data = {}
    let headers = {}

    let [authMod, authFormat] = botList.auth_format.split("#");

    if(authMod != "u" && authMod != "h" && authMod != "b") {
        throw new Error("Only u#, h#, and b# are supported for auth_format in bot_lists in config.yaml")
    }

    switch (authMod) {
        case "u": 
            if(url.includes("?")) {
                url += "&" + parseHeader(authFormat, variables)
            } else {
                url += "?" + parseHeader(authFormat, variables)
            }
            break;
        case "h":
            let [headerName, ...headerExt] = authFormat.split("/");
            console.log(headerName, headerExt)
            headers = {
                ...headers,
                [headerName]: parseHeader(headerExt.join(""), variables)
            }
            break;
        case "b":
            let [bodyName, ...bodyExt] = authFormat.split("=");
            data = {
                ...data,
                [bodyName]: parseHeader(bodyExt.join(""), variables)
            }
            break;
    }

    if(action.data_format) {
        for (const key in action.data_format) {
            let value = action.data_format[key].split("|")
            if(value.length == 0) {
                value.push("") // Handle simple variable substitution
            }

            let variable = variables[value[0]]
            
            if(variable) {
                switch (value[1]) {
                    case "int":
                        variable = parseInt(variable)
                        break;
                    case "float":
                        variable = parseFloat(variable)
                        break;
                    case "bool":
                        variable = variable == "true"
                        break;
                }
                data[key] = variable
            }
        }
    }

    return {
        url,
        data,
        headers
    }
}

export async function postStats(client: AntiRaid, botList: BotList, action: BotListAction) {
    let variables = {
        servers: await getServerCount(client),
        shards: await getShardCount(client),
        members: await getServerCount(client),
        botId: client.user.id,
    }

    let { url, data, headers } = createData(botList, action, variables)

    client.logger.info("Posting stats", { url, data, headers })

    let res = await fetch(url, {
        method: action.method,
        headers: {
            ...headers,
            "Content-Type": "application/json"
        },
        body: Object.keys(data).length > 0 ? JSON.stringify(data) : undefined
    })

    return res
}