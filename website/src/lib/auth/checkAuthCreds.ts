import { get } from '../configs/functions/services'
import { fetchClient } from '../fetch/fetch'
import logger from '../ui/logger'
import { LSAuthData } from './core'

export const checkAuthCreds = async (data: LSAuthData) => {
    // Check that the token is valid
    const testAuthResp = await fetchClient(`${get('splashtail')}/auth/test`, {
        method: 'POST',
        body: JSON.stringify({
            auth_type: 'user',
            target_id: data.user_id,
            token: data.token
        })
    })

    if (!testAuthResp.ok) {
        return false
    }

    logger.info('Auth', 'Auth token is valid!')
    return true
}
