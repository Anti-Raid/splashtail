<!--
From Infinity-Next commit 5317beadeeb039afe94f0a3027424c6322e64d40 

filename: src/components/layouts/Button.tsx

Converted to SvelteKit from NextJS for panel use

=> Link button that opens in a new tab
-->

<script lang="ts">
	import ButtonInner from './ButtonInner.svelte';
	import type { Color } from './colors';

	let className: string = '';
	export { className as class };
	export let color: Color;
	export let icon: string;
	export let text: string;
	export let href: string;
</script>

<a {href} target="_blank" rel="noreferrer">
	<ButtonInner {color} {icon} {text} class={className} />
</a>
