import logger from '../ui/logger'
import { LSAuthData } from './core'

export const getAuthCreds = () => {
    logger.info('Auth', 'Loading auth data')

    let token = localStorage.getItem('wistala')

    let data: LSAuthData | null = null

    if (token) {
        try {
            data = JSON.parse(token)
            if (data?.expiresOn && data?.expiresOn < Date.now()) {
                logger.info('Auth', 'Auth data expired')
                localStorage.removeItem('wistala')
                return null
            }

            if (!data?.user_id || !data?.token) {
                return null
            }

            return data
        } catch (err) {
            // User is not logged in
            logger.error('Layout', 'Auth data invalid', err)
        }
    }

    return null
}
