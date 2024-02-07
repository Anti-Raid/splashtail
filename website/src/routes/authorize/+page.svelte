<script lang="ts">
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
	import { ApiError, AuthorizeRequest, UserLogin } from "$lib/generated/types";
	import logger from "$lib/ui/logger";
	import Message from "../../components/Message.svelte";

    const createSession = async () => {
        let searchParams = new URLSearchParams(window.location.search);

        if(!searchParams.has("code")) {
            throw new Error("No code in URL")
        }

        let guildId = searchParams.get("guild_id") // Given if user has invited bot using full auth flow

        let json: AuthorizeRequest = {
            protocol: "a1",
            scope: "normal",
            code: searchParams.get("code") || "",
            redirect_uri: `${window.location.origin}/authorize`
        }
        let res = await fetchClient(`${get('splashtail')}/oauth2`, {
            method: "POST",
            body: JSON.stringify(json)
        })

        if(!res.ok) {
            let err: ApiError = await res.json()
            throw new Error(err?.message?.toString() || "Unknown error creating session")
        }

        let data: UserLogin = await res.json()

        localStorage.setItem("wistala", JSON.stringify(data))

        setTimeout(() => {
            if(guildId) {
                window.location.href = `/dashboard/guilds?id=${guildId}`
            } else {
                if(searchParams?.get("state")) {
                    try {
                        let path = atob(searchParams?.get("state") || "")

                        window.location.href = path
                        return
                    } catch(e) {
                        logger.error("Failed to redirect to state path", e)
                    }
                }

                window.location.href = "/dashboard"
            }
        }, 1000)
        
        return data
    }
</script>

{#await createSession()}
    <Message type="loading-big">
        Authorizing...
    </Message>
{:then}
    <Message type="success">
        Authorized!
    </Message>
{:catch error}
    <Message type="error">
        {error?.message || "Unknown error creating session"}
    </Message>
{/await}