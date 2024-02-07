import support from '$lib/configs/data/support.json'

export const loginUser = async () => {
    let state = btoa(window.location.toString())

    window.location.href = support.invite.no_bot.replace("{permissions}", support.invite.permissions)
    .replace("{client_id}", support.invite.client_id)
    .replace("{redirect_url}", `${window.location.origin}/authorize`)
    .replace("{state}", state)
}