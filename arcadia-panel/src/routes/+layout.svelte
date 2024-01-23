<script lang="ts">
	import '../app.postcss';
	import '$lib/styles/global.css';
	import '$lib/styles/mainsite/customColors.css';
	import '$lib/styles/mainsite/global.css';
	import { SvelteToast } from '@zerodevx/svelte-toast';
	import Menubar from '../components/Menubar.svelte';
	import { onMount } from 'svelte';

	const options = {};

	const buildInfo = {
		// @ts-ignore
		nodeEnv: I_NODE_ENV,
		// @ts-ignore
		publicCommit: I_COMMIT,
		// @ts-ignore
		lastMod: I_LAST_MOD,
		// @ts-ignore
		version: I_VERSION
	};

	onMount(async () => {
		const Sentry = await import('@sentry/browser');
		Sentry.init({
			dsn: 'https://8d6d3598571136c2a6c7dcba71ca0363@trace.infinitybots.gg/5',
			tunnel: `https://spider.infinitybots.gg/failure-management?to=br0`,
			replaysSessionSampleRate: 0.3,
			tracesSampleRate: 0.4,
			integrations: [new Sentry.Replay()],
			release: `panelv2@${buildInfo?.version}-${buildInfo?.publicCommit})`
		});
	});
</script>

<div data-theme="violet" class="flex min-h-screen flex-col justify-between overflow-x-hidden">
	<header class="mt-1">
		<Menubar />
	</header>

	<main class="text-white bg-contain">
		<slot />
		<SvelteToast {options} />
	</main>

	<footer class="mb-auto border-white border-t-2">
		<p class="text-center text-white text-md font-semibold">&copy; 2020 Infinity Development</p>
		<small class="text-center text-white text-sm font-semibold">
			{buildInfo?.version}-{buildInfo?.publicCommit}-{buildInfo?.nodeEnv?.substring(0, 4)} ({buildInfo?.lastMod})
		</small>
	</footer>
</div>

<style>
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		width: 100%;
		margin: 0 auto;
		box-sizing: border-box;
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 12px;
	}

	@media (min-width: 480px) {
		footer {
			padding: 12px 0;
		}
	}
</style>
