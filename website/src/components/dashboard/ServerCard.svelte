<script lang="ts">
	import Icon from '@iconify/svelte';
import { onMount } from 'svelte';

	interface Action {
		name: string;
		icon: string;
		click?: () => void;
		href?: string;
	}

	export let id: string;
	export let name: string;
	export let image: string | null;
	export let blurImage: boolean = false;
	export let actions: Action[] = [];
	export let mainAction: Action;
	export let disabled: string = ""; // If set, disabled with this message

	const imageLoadError = (a: any) => {
		a.target.src = '/logo.webp';
	};

	// CSS Gradient Generator
	let hexString = '0123456789abcdef';

	let randomColor = () => {
		let hexCode = '#';

		for (let i = 0; i < 6; i++) {
			hexCode += hexString[Math.floor(Math.random() * hexString.length)];
		}

		return hexCode;
	};

	let generateGrad = (p: any) => {
		let colorOne = randomColor();
		let colorTwo = randomColor();
		let angle = Math.floor(Math.random() * 360);

		p.style.background = `linear-gradient(${angle}deg, ${colorOne}, ${colorTwo})`;
	};

	onMount(() => {
		const p: HTMLElement | null = document.getElementById(`${id}_banner`);
		if (p) generateGrad(p);
	});
</script>

<section class="rounded-lg p-2 card bg-themable-600/50 shadow-white/50 text-white">
	<div class="h-16 rounded-t" id="{id}_banner" />

	<div class="bg-slate-900 dark:bg-gray-800 pt-6 px-6 pb-2 rounded-b">
		<div class="flex justify-center items-center mb-2">
			<img
				class={"h-7 w-7 rounded-md icon " + (blurImage ? "blur" : "")}
				src={image}
				height="28px"
				width="28px"
				alt="{name}'s Server Image"
				on:error={imageLoadError}
			/>

			{#if disabled}
				<span class="ml-2 text-xl font-extrabold dark:text-white truncate">{name}</span>
			{:else}
				{#if mainAction.click}
					<button on:click={mainAction.click} class="ml-2 text-xl font-extrabold dark:text-white truncate hover:underline">
						{name}
					</button>
				{:else}
					<a href={mainAction.href} class="ml-2 text-xl font-extrabold dark:text-white truncate hover:underline block">
						{name}
					</a>
				{/if}
			{/if}
		</div>

		{#if disabled}
			<p class="font-extrabold dark:text-white text-red-500 h-16 max-h-16">{disabled}</p>
		{:else if $$slots.message}
			<div class="h-16 max-h-16">
				<slot name="message" />
			</div>
		{/if}

		<div class="buttons flex flex-col justify-center items-center space-x-2 text-lg">
			{#if disabled}
				<button
					class="mt-3 bg-indigo-300 px-4 py-3 text-white rounded-md font-medium hover:cursor-not-allowed"
					disabled={true}
					aria-disabled="true"
				>
					<Icon icon="mdi:lock" class="mr-2 inline" />
					{mainAction.name}
				</button>
			{:else}
				{#if mainAction.click}
					<button
						on:click={mainAction.click}
						class="mt-3 bg-indigo-600 px-4 py-3 text-white rounded-md font-medium hover:cursor-pointer hover:bg-indigo-400"
					>
						<Icon icon={mainAction.icon} class="mr-2 inline" />
						{mainAction.name}
					</button>
				{:else}
					<a
						href={mainAction.href}
						class="mt-3 bg-indigo-600 px-4 py-3 text-white rounded-md font-medium hover:cursor-pointer hover:bg-indigo-400 block"
					>
						<Icon icon={mainAction.icon} class="mr-2 inline" />
						{mainAction.name}
					</a>
				{/if}

				{#each actions as action}
					{#if action.click}
						<button
							on:click={action.click}
							class="mt-3 bg-indigo-600 px-4 py-2 text-white rounded-md font-medium hover:cursor-pointer hover:bg-indigo-400"
						>
							<Icon icon={action.icon} class="mr-2 inline" />
							{action.name}
						</button>
					{:else}
						<a
							href={action.href}
							class="mt-3 bg-indigo-600 px-4 py-2 text-white rounded-md font-medium hover:cursor-pointer hover:bg-indigo-400 block"
						>
							<Icon icon={action.icon} class="mr-2 inline" />
							{action.name}
						</a>
					{/if}
				{/each}
			{/if}
		</div>
	</div>
</section>

<style>
.icon.blur:hover {
        filter: blur(0px);
}

.card {
        transition: all 0.3s;
        box-shadow:
                0 4px 8px 0 #23272a,
                0 6px 20px 0 rgba(0, 0, 0, 0.19);
}

.card:hover {
        transform: translate(0px, -5px);
}
</style>