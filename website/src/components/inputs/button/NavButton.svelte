<script lang="ts">
    export let current: boolean;
    export let title: string;
    export let href: string = "";
    export let onClick: (() => void) | undefined = undefined;
    export let disabled: boolean = false;
    export let extClass: string = "";

    let classes: string;

    $: {
        let baseClasses = current 
        ? "px-4 py-2 text-sm font-medium text-left text-gray-50 rounded-lg cursor-pointer bg-slate-700 focus:outline-none focus:ring-1 focus:ring-inset focus:ring-white" 
        : "px-4 py-2 text-sm font-medium text-left text-gray-300 transition-colors duration-150 bg-transparent rounded-lg cursor-pointer hover:bg-slate-800 hover:text-gray-50 focus:outline-none focus:ring-1 focus:ring-inset focus:ring-white"

        classes = disabled ? baseClasses + " opacity-50 cursor-not-allowed" : baseClasses;

        if (extClass) classes += " " + extClass;
    }
</script>

{#if href}
    <a aria-current={current ? 'page' : undefined} href={href} on:click={onClick} class={classes}>
        {title}
    </a>
{:else}
    <button aria-current={current} disabled={disabled} on:click={onClick} class={classes}>
        {title}
    </button>
{/if}

