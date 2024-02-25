<!--
From Infinity-Next commit 8cd768e5abb0a00889fcf8309e37fbbe93613c2c 

filename: src/components/layouts/Button.tsx

Converted to SvelteKit from NextJS for panel use

=> Tab buttons
-->

<script lang="ts">
	export let visible: boolean;
	export let text: string;
	export let onClick: () => void;
</script>

<button
	type="button"
	on:click={(evt) => {
		if (evt && evt.preventDefault) evt.preventDefault();

		onClick();
	}}
	class={visible
		? 'text-white p-2 mr-5 mb-2 border-b-4 hover:opacity-80'
		: 'text-white p-2 mr-5 mb-2 border-b-2 hover:opacity-80'}
>
	<span class={visible ? 'opacity-60' : ''}>{text}</span>
</button>
