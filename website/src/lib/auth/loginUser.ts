import { get } from "$lib/configs/functions/services"
import { fetchClient } from "$lib/fetch/fetch"
import { ApiError, OauthMeta } from "$lib/generated/types"

export const loginUser = async () => {
    let res = await fetchClient(`${get('splashtail')}/oauth2/meta`)

    if(!res.ok) {
        let err: ApiError = await res.json()

        throw new Error(err.message?.toString() || "An error occurred while logging in")
    }

    let resp: OauthMeta = await res.json()

    window.location.href = (`${resp?.oauth2_base}?client_id=${resp?.client_id}&scope=${resp?.scopes?.join('%20')}&response_type=code&redirect_uri=${window.location.origin}/authorize`)
}