<script lang="ts">
	import { checkAuthCreds } from "$lib/auth/checkAuthCreds";
	import { LSAuthData } from "$lib/auth/core";
	import { getAuthCreds } from "$lib/auth/getAuthCreds";
	import { loginUser } from "$lib/auth/loginUser";
	import { logoutUser } from "$lib/auth/logoutUser";
	import Message from "../Message.svelte";

    let currentState: string = "Checking your credentials"

    const checkAuth = async (authCreds: LSAuthData) => {
        // Check auth
        if(!authCreds) {
            throw new Error("No auth credentials found")
        }

        try {
            return await checkAuthCreds(authCreds);
        } catch {
            return true
        }
    }

    const checkAuthData = async () => {
        let authCreds = getAuthCreds();

        if(!authCreds) {
            currentState = "Logging you in...."
            
            try {
                await loginUser()
            } catch (err) {
                throw new Error(err?.toString() || "Failed to login")
            }

            return false
        }

        let r = await checkAuth(authCreds)

        if(!r) {
            currentState = "Session expired..."
            logoutUser()
            window.location.reload()
            return false
        }

        setInterval(async () => {
            if(!authCreds) return;
            let r = await checkAuth(authCreds)

            if(!r) {
                logoutUser()
                window.location.reload()
                return
            }
        }, 1000 * 60 * 5)

        return true
    }
</script>

{#await checkAuthData()}
    <Message
        type="loading"
    >
        Loading dashboard
    </Message>
    <small>
        <span class="font-semibold">Current State: </span>
        {currentState}
    </small>
{:then res}
    {#if res}
        <slot />
    {:else}
        <Message
            type="loading"
        >
            Please wait...
        </Message>
    {/if}
{:catch error}
    <Message
        type="error"
    >
        {error?.toString() || "Failed to load dashboard"}
    </Message>
{/await}