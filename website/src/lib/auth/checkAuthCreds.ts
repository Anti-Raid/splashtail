import { get } from '../configs/functions/services'
import { fetchClient } from '../fetch/fetch'
import logger from '../ui/logger'
import { LSAuthData } from './core'

interface AuthData {
    authorized: boolean
    banned: boolean
    data: {
        session_id: string,
        perm_limits: string[]
    },
    id: string
    target_type: string
}

export let testAuthData: AuthData | null = null

export const checkAuthCreds = async (data: LSAuthData) => {
    // Check that the token is valid
    const testAuthResp = await fetchClient(`${get('splashtail')}/auth/test`, {
        method: 'POST',
        body: JSON.stringify({
            auth_type: 'User',
            target_id: data.user_id,
            token: data.token
        })
    })

    if (testAuthResp.status == 401) {
        return false
    } else if (!testAuthResp.ok) {
        throw new Error('An error occurred while checking auth credentials')
    }

    logger.info('Auth', 'Auth token is valid!')

    testAuthData = await testAuthResp.json()

    return true
}
