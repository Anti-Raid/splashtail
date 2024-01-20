import services from '../data/services.json'

type ConfigKeys = keyof typeof services

export const get = (key: ConfigKeys): string => {
    if (process.env.BUILD_ENV == 'production') {
        return services?.[key]?.production
    }

    if (process.env[`${key.toUpperCase()}_OVERRIDE`]) {
        return process.env[`${key.toUpperCase()}_OVERRIDE`] as string
    }

    // @ts-ignore
    return services?.[key]?.development || services?.[key]?.production
}

