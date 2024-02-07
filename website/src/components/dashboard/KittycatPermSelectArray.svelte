<script lang="ts">
	import KittycatPermSelector from "./KittycatPermSelector.svelte";

    export let perms: string[] = [];
    export let id: string;
</script>

<section id={id} class="kittycat-perm-select-array mb-2">
    {#if perms?.length == 0}
    <button class="bg-themable-600 hover:bg-themable-400 p-3" on:click={() => {
        perms = [...perms, ''];
    }}>Add Permission</button>
{/if}

{#each perms as perm, i}
    <KittycatPermSelector bind:perm={perm} />

    <div class="flex flex-row">
        <button class="bg-themable-600 hover:bg-themable-400 p-3 mr-2" on:click={() => {
            perms = [...perms.slice(0, i + 1), '', ...perms.slice(i + 1)];
        }}>Add Permission</button>
        <button class="bg-red-600 hover:bg-red-400 p-3" on:click={() => {
            perms = perms.filter((_, index) => index !== i);
        }}>Remove Permission</button>
    </div>
{/each}
</section>
