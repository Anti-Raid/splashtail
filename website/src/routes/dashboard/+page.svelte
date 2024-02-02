<script lang="ts">
	import { getAuthCreds } from "$lib/auth/getAuthCreds";
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
	import { DashboardGuildData } from "$lib/generated/types";
    let currentState = "Loading dashboard data"
    import Message from "../../components/Message.svelte";
	import ServerCard from "../../components/dashboard/ServerCard.svelte";
    import Column from "../../components/Column.svelte";

    let guilds: DashboardGuildData;

    const loadIndexDashPage = async () => {
        let authCreds = getAuthCreds();

        if(!authCreds) throw new Error("No auth credentials found")

        currentState = "Fetching user guild list"

        let res = await fetchClient(`${get('splashtail')}/users/${authCreds?.user_id}/guilds`, {
            auth: authCreds?.token
        })

        if(!res.ok) throw new Error("Failed to fetch user guild list")

        guilds = await res.json()
    }
</script>

{#await loadIndexDashPage()}
    <Message
        type="loading"
    >
        Loading dashboard
    </Message>
    <small>
        <span class="font-semibold">Current State: </span>
        {currentState}
    </small>
{:then}
        <Column>
            {#each (guilds?.guilds || []) as guild}
                <ServerCard 
                    id={guild?.id || ""} 
                    name={guild?.name || ""} 
                    image={guild?.avatar || "/logo.webp"} 
                    mainAction={{name: "Invite", href: `/invite/${guild?.id}`, icon: "mdi:discord"}}
                />
            {/each}
        </Column>
{:catch error}
    <Message
        type="error"
    >
        {error?.toString() || "Failed to load dashboard"}
    </Message>
{/await}