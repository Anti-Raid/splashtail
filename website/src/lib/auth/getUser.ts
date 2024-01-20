import { get } from '../configs/functions/services'
import { fetchClient } from '../fetch/fetch'
import { User } from '../generated/types'
import logger from '../ui/logger'
import { LSAuthData } from './core'

let cachedUserValue: User | null = null

export const getUser = async (data: LSAuthData) => {
    if(cachedUserValue) {
        return cachedUserValue
    }

    const res = await fetchClient(`${get('splashtail')}/users/${data.user_id}`)

    if (!res.ok && data?.user_id) {
        logger.error('Auth', 'Could not find user perm information from API')
        return
    }

    let userData: User = await res.json()

    logger.info('Layout', 'Got user information from API', userData)

    cachedUserValue = userData

    return userData
}
