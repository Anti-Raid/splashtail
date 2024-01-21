import services from '../data/services.json'
import { PUBLIC_BUILD_ENV } from '$env/static/public';

type ConfigKeys = keyof typeof services

export const get = (key: ConfigKeys): string => {
    if (PUBLIC_BUILD_ENV == 'produ:qction') {
        return services?.[key]?.production
    }

    // @ts-ignore
    return services?.[key]?.development || services?.[key]?.production
}

