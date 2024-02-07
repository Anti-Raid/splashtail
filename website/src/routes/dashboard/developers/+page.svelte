<script lang="ts">
	import { testAuthData } from "$lib/auth/checkAuthCreds";
	import { getAuthCreds } from "$lib/auth/getAuthCreds";
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
	import { ApiError, UserSession, UserSessionList } from "$lib/generated/types";
	import { error, success } from "$lib/toast";
    import Message from "../../../components/Message.svelte";
    import { DataHandler, Datatable, Th, ThFilter } from "@vincjo/datatables";
	import { Readable } from "svelte/store";

    let sessionRows: Readable<UserSession[]>;
    let otherSessionRows: Readable<UserSession[]>;

    let currentState = "Loading developer portal"

    const loadGuildData = async () => {
        let authCreds = getAuthCreds();

        if(!authCreds) throw new Error("No auth credentials found")

        currentState = "Fetching session data"

        let res = await fetchClient(`${get('splashtail')}/users/${authCreds?.user_id}/sessions`, {
            auth: authCreds?.token,
            onRatelimit: (n, err) => {
                if(!n) {
                    currentState = "Retrying fetching of session data"
                } else {
                    currentState = `${err?.message} [retrying again in ${n/1000} seconds]`
                }
            },
        })

        if (!res.ok) {
            if(!res.ok) {}
            let err: ApiError = await res.json()
            throw new Error(`Failed to fetch base session data: ${err?.message} (${err?.context})`)
        }

        let data: UserSessionList = await res.json()

        const sessionHandler = new DataHandler(data.sessions.filter(f => f?.type == 'login') as UserSession[], { rowsPerPage: 20 })
        const otherSessionHandler = new DataHandler(data.sessions.filter(f => f?.type != 'login') as UserSession[], { rowsPerPage: 20 })

        sessionRows = sessionHandler.getRows()
        otherSessionRows = otherSessionHandler.getRows()

        return {
            otherSessionHandler,
            sessionHandler,
        }
    }

    const revokeSession = async (sessionId: string) => {
        try {
            let authCreds = getAuthCreds();

            if(!authCreds) throw new Error("No auth credentials found")

            let res = await fetchClient(`${get('splashtail')}/users/${authCreds?.user_id}/sessions/${sessionId}`, {
                method: "DELETE",
                auth: authCreds?.token
            })

            if(res.ok) {
                success(`Successfully revoked session ${sessionId}`)
            } else {
                let err: ApiError = await res.json()
                error(`Failed to revoke session: ${err?.message} (${err?.context})`)
            }
         } catch (err) {
            error(`Failed to revoke session: ${err}`)
        }
    }
</script>

{#await loadGuildData()}
    <Message
        type="loading"
    >
        Loading dashboard
    </Message>
    <small>
        <span class="font-semibold">Current State: </span>
        {currentState}
    </small>
{:then data}
    <h2 class="font-semibold text-2xl">Your Sessions</h2>

    <p>
        <strong>
            Note: Session Tokens (created by logging in) expire every 1 hour and are not suitable 
            for developing on the Anti-Raid API. Please create an API token instead for that!
        </strong><br/><br/>

        Be sure to revoke sessons you don't recognize! The ID of the session you are currently logged onto is 
        <em class="opacity-70">{testAuthData?.data?.session_id}</em>
    </p>

    <Datatable handler={data.sessionHandler} search={false}>
        <table class="overflow-x-auto">
            <thead>
                <tr>
                    <Th handler={data.sessionHandler} orderBy={"id"}>ID</Th>
                    <Th handler={data.sessionHandler} orderBy={"name"}>Name</Th>
                    <Th handler={data.sessionHandler} orderBy={"type"}>Type</Th>
                    <Th handler={data.sessionHandler} orderBy={"expiry"}>Expiry</Th>
                    <Th handler={data.sessionHandler} orderBy={"created_at"}>Created At</Th>
                    <Th handler={data.sessionHandler} orderBy={"id"}>Actions</Th>
                </tr>
                <tr>
                    <ThFilter handler={data.sessionHandler} filterBy={"id"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"name"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"type"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"expiry"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"created_at"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"id"} />
                </tr>
            </thead>
            <tbody>
                {#each $sessionRows as session (session.id)}
                    <tr>
                        <td>
                            {session.id}

                            {#if session.id == testAuthData?.data?.session_id}
                                <span class="text-green-500"> (Current Session)</span>
                            {/if}
                        </td>
                        <td>{session.name || "Unnamed Session"}</td>
                        <td>{session.type}</td>
                        <td>{session.expiry}</td>
                        <td>{session.created_at}</td>
                        <td>
                            <button 
                                class="text-red-400 hover:text-red-600"
                                on:click={() => revokeSession(session.id)}
                            >  
                                Revoke
                            </button>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </Datatable>

    <h1 class="font-semibold text-2xl">API Tokens</h1>

    <Datatable handler={data.otherSessionHandler} search={false}>
        <table class="overflow-x-auto">
            <thead>
                <tr>
                    <Th handler={data.otherSessionHandler} orderBy={"id"}>ID</Th>
                    <Th handler={data.otherSessionHandler} orderBy={"name"}>Name</Th>
                    <Th handler={data.otherSessionHandler} orderBy={"type"}>Type</Th>
                    <Th handler={data.otherSessionHandler} orderBy={"expiry"}>Expiry</Th>
                    <Th handler={data.otherSessionHandler} orderBy={"created_at"}>Created At</Th>
                    <Th handler={data.otherSessionHandler} orderBy={"id"}>Actions</Th>
                </tr>
                <tr>
                    <ThFilter handler={data.otherSessionHandler} filterBy={"id"} />
                    <ThFilter handler={data.otherSessionHandler} filterBy={"name"} />
                    <ThFilter handler={data.otherSessionHandler} filterBy={"type"} />
                    <ThFilter handler={data.otherSessionHandler} filterBy={"expiry"} />
                    <ThFilter handler={data.otherSessionHandler} filterBy={"created_at"} />
                    <ThFilter handler={data.otherSessionHandler} filterBy={"id"} />
                </tr>
            </thead>
            <tbody>
                {#each $otherSessionRows as session (session.id)}
                    <tr>
                        <td>{session.id}</td>
                        <td>{session.name || "Unnamed API Token"}</td>
                        <td>{session.type}</td>
                        <td>{session.expiry}</td>
                        <td>{session.created_at}</td>
                        <td>
                            <button 
                                class="text-red-400 hover:text-red-600"
                                on:click={() => revokeSession(session.id)}
                            >  
                                Revoke
                            </button>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </Datatable>
{:catch err}
    <Message type="error">Error loading dashboard data: {err}</Message>
{/await}

<style>
    table {
            color: white;
            margin: 0 !important;
    }
    tbody td {
            border: 1px solid #f5f5f5;
            padding: 4px 20px;
    }
    tbody tr {
            transition: all, 0.2s;
    }
    tbody tr:hover {
            background: #252323;
    }
</style>
