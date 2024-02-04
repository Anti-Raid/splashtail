import { ApiError } from '$lib/generated/types'
import logger from '../ui/logger'

interface FetchClientOptions extends RequestInit {
    auth?: string
    noExtraHeaders?: boolean
    noWait?: boolean
    onRatelimit?: (n: number, err: ApiError) => void
}

export async function fetchClient(url: string, options?: FetchClientOptions): Promise<Response> {
    let rawOptions = options

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
    }

    let modifier = ''

    if (options.auth) {
        // @ts-ignore
        headers['Authorization'] = `User ${options.auth}`
        modifier += ' (authorized)'
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

        if (res.headers.get("Retry-After") && !options?.noWait) {
            logger.info("FetchClient", "Rate limited", res.headers.get('Retry-After'), res.headers)
            let retryAfter = res.headers.get('Retry-After')

            if (retryAfter) {
                let err: ApiError = await res.json()

                let n = parseFloat(retryAfter || "3") * 1000

                if (options.onRatelimit) {
                    options.onRatelimit(n, err)
                }

                // Wait for the time specified by the server
                if (!options.noWait) {
                    logger.info('FetchClient', `Rate limited, waiting ${retryAfter} seconds`)
                    await new Promise(resolve => setTimeout(resolve, n))

                    if (options.onRatelimit) {
                        options.onRatelimit(0, err)
                    }    

                    return await fetchClient(url, rawOptions)
                }
            }
        }

        return res
    } catch (err) {
        logger.error('FetchClient', 'Error', err)
        throw err
    }
}

