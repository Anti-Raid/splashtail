<script lang="ts">
	import { onMount } from 'svelte';

	interface ButtonType {
		name: String;
		click: () => void;
	}

	export let name: string;
	export let title: string;
	export let image: string | null;
	export let button: ButtonType;

	const ImageLoadError = (a: any) => {
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
		const p: HTMLElement | null = document.getElementById(`${title}_banner`);
		if (p) generateGrad(p);
	});
</script>

<div class="inline-grid">
	<button class="cursor-default" on:click={button.click}>
		<div class="flex h-16 max-w-sm rounded-t" id="{title}_banner" />

		<div class="max-w-sm bg-white dark:bg-gray-800 pt-6 px-6 pb-2 rounded-b">
			<div class="flex justify-center items-center mb-2">
				<img
					class="h-7 w-7 rounded-md"
					src={image}
					height="28px"
					width="28px"
					alt="{image}'s Server Image"
					on:error={ImageLoadError}
				/>

				<h3 class="ml-2 text-xl font-extrabold text-black dark:text-white truncate hover:underline">
					{title}
				</h3>
			</div>

			<button
				class="mt-3 bg-indigo-600 px-3 py-2 text-white rounded-md text-base font-medium hover:cursor-pointer hover:bg-indigo-400"
				><i class="fa-solid fa-plus pr-2" /> {button.name}</button
			>
		</div>
	</button>
</div>
