<script lang="ts">
	import { goto } from "$app/navigation";
	import { getAuthCreds } from "$lib/auth/getAuthCreds";
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
    import Message from "../../../components/Message.svelte";

    let currentState = "Loading dashboard data"

    const loadGuildData = async () => {
        let authCreds = getAuthCreds();

        if(!authCreds) throw new Error("No auth credentials found")

        let searchParams = new URLSearchParams(window.location.search);

        let guildId = searchParams.get("id");

        if(!guildId) {
            await goto("/dashboard")
            return
        }

        currentState = "Fetching guild data"

        let res = await fetchClient(`${get('splashtail')}/users/${authCreds?.user_id}/guilds/${guildId}`, {
            auth: authCreds?.token,
            onRatelimit: (n) => {
                if(!n) {
                    currentState = "Fetching guild data"
                } else {
                    currentState = `Ratelimited, retrying guild data fetch in ${n/1000} seconds`
                }
            }
        })

        return true
    }
</script>

{#await loadGuildData()}
    <Message type="loading">{currentState}</Message>
{:then r}
    {#if r}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"> 
        </div>
    {:else}
        <Message type="loading">Please wait</Message>
    {/if}
{:catch err}
    <Message type="error">Error loading dashboard data: {err}</Message>
{/await}