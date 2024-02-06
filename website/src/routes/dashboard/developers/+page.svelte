<script lang="ts">
	import { getAuthCreds } from "$lib/auth/getAuthCreds";
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
	import { ApiError, UserSession, UserSessionList } from "$lib/generated/types";
    import Message from "../../../components/Message.svelte";
    import { DataHandler, Datatable, Th, ThFilter } from "@vincjo/datatables";
	import { Readable } from "svelte/store";

    let sessionRows: Readable<UserSession[]>;

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

        sessionRows = sessionHandler.getRows()

        return {
            sessionHandler,
            rows: sessionRows
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

    <p>Be sure to revoke sessons you don't recognize!</p>

    <Datatable handler={data.sessionHandler} search={false}>
        <table class="overflow-x-auto">
            <thead>
                <tr>
                    <Th handler={data.sessionHandler} orderBy={"id"}>ID</Th>
                    <Th handler={data.sessionHandler} orderBy={"type"}>Type</Th>
                    <Th handler={data.sessionHandler} orderBy={"expiry"}>Expiry</Th>
                    <Th handler={data.sessionHandler} orderBy={"created_at"}>Created At</Th>
                </tr>
                <tr>
                    <ThFilter handler={data.sessionHandler} filterBy={"id"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"type"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"expiry"} />
                    <ThFilter handler={data.sessionHandler} filterBy={"created_at"} />
                </tr>
            </thead>
            <tbody>
                {#each $sessionRows as session (session.id)}
                    <tr>
                        <td>{session.id}</td>
                        <td>{session.type}</td>
                        <td>{session.expiry}</td>
                        <td>{session.created_at}</td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </Datatable>
{:catch err}
    <Message type="error">Error loading dashboard data: {err}</Message>
{/await}