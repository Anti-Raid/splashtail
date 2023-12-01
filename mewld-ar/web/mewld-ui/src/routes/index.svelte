<script>
    import { session } from "$app/stores";

    import { goto } from '$app/navigation';

    let instanceUrl = "";

    let statusMsg = ""

    async function login() {
        if(!instanceUrl.startsWith("http:")  && !instanceUrl.startsWith("https:")) {
            instanceUrl = "http://" + instanceUrl;
        }
        statusMsg = `Trying to connect to ${instanceUrl}...`;

        let loginRes = await fetch(`${instanceUrl}/login?api=api@http://${window.location.host}@${instanceUrl}`)

        if(!loginRes.ok) {
            statusMsg = `Could not connect to server. Got bad status of ${loginRes.statusText}`;
            return
        }

        let loginUrl = await loginRes.text()

        statusMsg = `Redirecting to ${loginUrl}...`;
        window.location.href = loginUrl;
    }

    async function logout() {
        window.location.href = "/ss/logout"
    }
</script>

{#if $session.maint}
    <div class="alert alert-danger">
        <button on:click={() => logout()}>Logout</button>
        The mewld instance you tried to connect to is currently under maintenance, please try again later
    </div>
{:else}
    {#if $session.id}
        <button on:click={() => logout()}>Logout</button>

        <h1>Homepage</h1>
        <button on:click={() => goto("/clusters")}>View Clusters</button>
    {:else}
        <h1>Instance Connect</h1>
        <input bind:value={instanceUrl} type="text" placeholder="Input mewld instance URL" />
        <button on:click={() => login()}>Connect</button>
        <p>{statusMsg}</p>
        <footer>
            <p>If connection does not work, try enabling <a href="https://experienceleague.adobe.com/docs/target/using/experiences/vec/troubleshoot-composer/mixed-content.html?lang=en">mixed content</a></p>
        </footer>
    {/if}
{/if}