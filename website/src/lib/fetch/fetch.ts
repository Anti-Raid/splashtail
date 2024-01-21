import logger from '../ui/logger'

interface FetchClientOptions extends RequestInit {
    auth?: string
    noExtraHeaders?: boolean
}

export async function fetchClient(url: string, options?: FetchClientOptions): Promise<Response> {
    if (!options) {
        options = {}
    }

    let headers = {}

    if (!options?.noExtraHeaders) {
        // @ts-ignore
        headers['Content-Type'] = 'application/json'
    }

    if (options.headers) {
        headers = {
            ...headers,
            ...options.headers
        }

        delete options.headers
    }

    let modifier = ''

    if (options.auth) {
        // @ts-ignore
        headers['Authorization'] = `User ${options.auth}`
        modifier += ' (authorized)'
        delete options.auth
    } else {
        // @ts-ignore
        if (headers['Authorization']) {
            logger.error('FetchClient', 'options.auth must be used for auth')
        }
    }

    logger.info('FetchClient', options.method ? options.method.toUpperCase() + modifier : 'GET' + modifier, url)

    try {
        const res = await fetch(url, {
            headers: headers,
            ...options
        })

        if ([408, 502, 503, 504].includes(res.status)) {
            throw new Error('Server currently undergoing maintenance')
        }

        return res
    } catch (err) {
        logger.error('FetchClient', 'Error', err)
        throw err
    }
}

