<script lang="ts">
    import Icon from '@iconify/svelte';

    export let icon: string;
    export let title: string;
</script>

<div class="relative">
    <dt>
        <div
            class="absolute flex h-12 w-12 items-center justify-center rounded-md bg-indigo-500 text-white"
        >
            <Icon icon={icon} class="text-2xl" />
        </div>
        <p class="ml-16 text-lg font-medium leading-6 text-white">{title}</p>
    </dt>
    <dd class="mt-2 ml-16 text-base text-white">
        <slot />
    </dd>
</div>
