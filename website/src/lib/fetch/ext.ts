import { get } from "$lib/configs/functions/services"
import { fetchClient } from "$lib/fetch/fetch"
import { InstanceList } from "$lib/generated/mewld/proc"
import { ApiError } from "$lib/generated/types"

let cachedData: Map<string, any> = new Map()

interface SharedRequester<T> {
    name: string
    requestFunc: () => Promise<T>
}

interface SharedRequestOpts {
    forceRefresh?: boolean
}

// Make a shared request also checking cache as well
export async function makeSharedRequest<T>(requester: SharedRequester<T>, opts?: SharedRequestOpts): Promise<T> {
    if(cachedData.has(requester.name) && !opts?.forceRefresh) {
        return cachedData.get(requester.name)
    }

    const data = await requester.requestFunc()

    cachedData.set(requester.name, data)

    return data
}

// Fetches the health of all clusters
export const opGetClusterHealth: SharedRequester<InstanceList> = {
    name: "clusterHealth",
    requestFunc: async (): Promise<InstanceList> => {
        const res = await fetchClient(`${get('splashtail')}/clusters/health`);
        if(!res.ok) {
            let resp: ApiError = await res.json()
            throw new Error(`Failed to fetch clusters health: ${res.status}: ${resp?.message}`)
        }
    
        const data: InstanceList = await res.json()
        
        return data
    }
}
