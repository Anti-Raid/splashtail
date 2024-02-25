<!--
From Infinity-Next commit 5317beadeeb039afe94f0a3027424c6322e64d40 

filename: src/components/layouts/Button.tsx

Converted to SvelteKit from NextJS for panel use

=> Reactive button 
-->

<script lang="ts">
	import { error } from '$lib/toast';
	import ButtonInner from './ButtonInner.svelte';
	import type { Color } from './colors';
	import type { States } from './states';

	let className: string = '';
	export { className as class };
	export let color: Color;
	export let icon: string;
	export let text: string;
	export let type: 'button' | 'submit' = 'submit';
	export let states: States;
	export let noRevertState: boolean = false;
	export let disableBtnAfter: string = '';
	export let onClick: () => Promise<boolean>;

	// Internal state
	enum ReactState {
		Normal,
		Loading,
		Clicked,
		Error
	}

	interface Display {
		icon: string;
		text: string;
		className: string;
		disabled: boolean;
	}

	let state: ReactState = ReactState.Normal; // Current state of the button
	let display: Display = { icon, text, className, disabled: false }; // Current display of the button

	$: {
		switch (state) {
			case ReactState.Normal:
				display = {
					...display,
					icon,
					text,
					className
				};
				break;
			case ReactState.Loading:
				display = {
					...display,
					icon: 'mdi:loading',
					text: states.loading,
					className: (className ? className + ' ' : '') + 'cursor-not-allowed animate-pulse'
				};
				break;
			case ReactState.Clicked:
				display = {
					...display,
					icon: 'mdi:check',
					text: states.success,
					className: className + display.disabled ? 'cursor-not-allowed animate-pulse' : ''
				};

				if (!disableBtnAfter) {
					display.disabled = false;
				} else {
					display = {
						...display,
						text: disableBtnAfter,
						icon: 'mdi:refresh',
						disabled: true
					};
				}

				break;
			case ReactState.Error:
				display = {
					...display,
					icon: 'mdi:alert-circle',
					text: states.error,
					className: className + display.disabled ? 'cursor-not-allowed animate-pulse' : ''
				};
				break;
		}
	}
</script>

<button
	class="w-full"
	disabled={display.disabled}
	aria-disabled={display.disabled}
	{type}
	on:click|preventDefault
	on:click={() => {
		display.disabled = true; // Disable the button
		if (state == ReactState.Loading) return;

		state = ReactState.Loading;

		setTimeout(() => {
			let resp = onClick().catch((e) => {
				error(`${e}`);
				state = ReactState.Error;

				// Wait 2 seconds
				if (!noRevertState && !disableBtnAfter) {
					setTimeout(() => {
						state = ReactState.Normal;
						display.disabled = false;
					}, 4000);
				}
			});

			// Check if Promise
			if (resp && resp.then) {
				resp.then((out) => {
					state = out ? ReactState.Clicked : ReactState.Error;

					// Wait 2 seconds
					if (!noRevertState && !disableBtnAfter) {
						setTimeout(() => {
							state = ReactState.Normal;
							display.disabled = false;
						}, 4000);
					}
				});
			} else {
				if (!noRevertState && !disableBtnAfter) {
					state = ReactState.Normal;
					display.disabled = false;
				}
			}
		}, 300);
	}}
>
	<ButtonInner {color} icon={display.icon} text={display.text} class={display.className} />
</button>
