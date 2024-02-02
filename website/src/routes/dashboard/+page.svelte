<script lang="ts">
	import { getAuthCreds } from "$lib/auth/getAuthCreds";
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
	import { DashboardGuildData } from "$lib/generated/types";
    let currentState = "Loading dashboard data"
    import Message from "../../components/Message.svelte";
	import ServerCard from "../../components/dashboard/ServerCard.svelte";
    import Column from "../../components/Column.svelte";
    import InputText from "../../components/inputs/InputText.svelte";
    import ButtonReact from "../../components/inputs/button/ButtonReact.svelte";
    import { Color } from "../../components/inputs/button/colors";

    let guilds: DashboardGuildData;
    let hasBot: string[] = [];

    let hasBotSearchFilter: string = "";
    let serverListSearchFilter: string = "";

    const loadIndexDashPage = async (refresh: boolean) => {
        let authCreds = getAuthCreds();

        if(!authCreds) throw new Error("No auth credentials found")

        currentState = "Fetching user guild list"

        let res = await fetchClient(`${get('splashtail')}/users/${authCreds?.user_id}/guilds?refresh=${refresh}`, {
            auth: authCreds?.token,
            onRatelimit: (n) => {
                if(!n) {
                    currentState = "Fetching user guild list"
                } else {
                    currentState = `Ratelimited, retrying user guild list fetch in ${n/1000} seconds`
                }
            }
        })

        if(!res.ok) throw new Error("Failed to fetch user guild list")

        guilds = await res.json()

        if(guilds?.has_bot) {
            hasBot = guilds?.has_bot;
        }
    }

    const recacheForce = async () => {
        try {
            await loadIndexDashPage(true)
            return true
        } catch (e) {
            return false
        }
    }
</script>

{#await loadIndexDashPage(false)}
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
    <h1 class="text-white font-semibold text-2xl">Servers With AntiRaid ({hasBot?.length})</h1>
    <p class="mb-2 text-red-500">You may or may not have permission to view or modify these servers...</p>

    <InputText 
        id="hasBotSearchFilter"
        label="Search for a server"
        placeholder="Server Name"
        bind:value={hasBotSearchFilter}
        minlength={0}
        showErrors={false}
    />

    <hr class="my-3" />

    <Column size="small">
        {#each (guilds?.guilds || [])?.filter(g => hasBot?.includes(g?.id || "") && (!hasBotSearchFilter || g?.name?.toLocaleLowerCase()?.includes(hasBotSearchFilter?.toLocaleLowerCase()))) as guild}
            <ServerCard 
                id={guild?.id || ""} 
                name={guild?.name || ""} 
                image={guild?.avatar || "/logo.webp"} 
                mainAction={
                    hasBot.includes(guild?.id || "") 
                    ? {name: "View", href: `/dashboard/guild/${guild?.id}`, icon: "mdi:discord"}
                    : {name: "Invite", href: `/dashboard/invite/${guild?.id}`, icon: "mdi:discord"}
                }
            />
        {/each}
    </Column>

    <hr class="my-5" />

    <h1 class="text-white font-semibold text-2xl mt-5 mb-2">Your Server List ({guilds?.guilds?.length})</h1>

    <InputText 
        id="serverListSearchFilter"
        label="Search for a server"
        placeholder="Server Name"
        bind:value={serverListSearchFilter}
        minlength={0}
        showErrors={false}
    />

    <Column size="small">
        {#each (guilds?.guilds || [])?.filter(g => !serverListSearchFilter || g?.name?.toLocaleLowerCase()?.includes(serverListSearchFilter?.toLocaleLowerCase())) as guild}
            <ServerCard 
                id={guild?.id || ""} 
                name={guild?.name || ""} 
                image={guild?.avatar || "/logo.webp"} 
                mainAction={
                    hasBot.includes(guild?.id || "") 
                    ? {name: "View", href: `/dashboard/guild/${guild?.id}`, icon: "mdi:discord"}
                    : {name: "Invite", href: `/dashboard/invite/${guild?.id}`, icon: "mdi:discord"}
                }
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

<ButtonReact 
    color={Color.Themable}
    text="Refresh Server List"
    icon="mdi:refresh"
    onClick={recacheForce}
    states={
        {
            loading: "Refreshing...",
            error: "Failed to refresh",
            success: "Refreshed"
        }
    }
/>