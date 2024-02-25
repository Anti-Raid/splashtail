<!--
From Infinity-Next commit 5317beadeeb039afe94f0a3027424c6322e64d40 

filename: src/components/layouts/Button.tsx

Converted to SvelteKit from NextJS for panel use

=> Inner button common code
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Color, getColor } from './colors';
	import Icon from '@iconify/svelte';

	let className: string = '';
	export { className as class };
	export let color: Color;
	export let icon: string;
	export let text: string;
	export let normalCursor: boolean = false; // Applies cursor-default hover:cursor-text

	// Internal state
	interface IState {
		colorClass: string;
		iconClass: string;
		extraClasses: string;
	}

	let state: IState | undefined;

	onMount(() => {
		let [colorClass, iconClass] = getColor(color);

		state = {
			colorClass,
			iconClass,
			extraClasses: normalCursor ? 'cursor-default hover:cursor-text' : ''
		};
	});
</script>

<div
	class={'flex items-center w-full text-left' +
		(className ? ' ' + className : '') +
		(state?.extraClasses ? ' ' + state?.extraClasses : '')}
	aria-live="polite"
>
	<div
		class={(state?.iconClass ? state?.iconClass + ' ' : '') +
			' px-4 py-2 rounded-l-lg text-white align-middle'}
	>
		<Icon
			{icon}
			inline={true}
			class={'inline-flex text-white align-items items-center justify-center' +
				(icon == 'mdi:loading' ? ' animate-spin' : '')}
		/>
	</div>
	<div
		class={(state?.colorClass ? state?.colorClass + ' ' : '') +
			' w-full px-4 py-2 rounded-r-lg text-white text-left'}
	>
		<p class="line-clamp-1">{text}</p>
	</div>
</div>
