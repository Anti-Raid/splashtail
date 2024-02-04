import { redirect } from '@sveltejs/kit'
import support from '$lib/configs/data/support.json'

export const load = async ({ url }) => {
    let basic = url.searchParams.get("basic")

    let urlBase = basic ? support.invite.basic : support.invite.full
    if(url.searchParams.get("guild_id")) {
        urlBase = `${urlBase}&guild_id=${url.searchParams.get("guild_id")}`
    }

    urlBase = urlBase.replace("{permissions}", support.invite.permissions)
    .replace("{client_id}", support.invite.client_id)
    .replace("{redirect_url}", `${url.origin}/authorize`)

    redirect(303, urlBase)
}