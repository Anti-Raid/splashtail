import { ApiError } from "$lib/generated/types";

export const formatApiError = (base: string, err: ApiError) => {
    if(err?.context) {
        return `${base}: ${err.message} [${err.context}]`;
    } else {
        return `${base}: ${err.message}`
    }
}