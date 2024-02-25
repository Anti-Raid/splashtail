<script lang="ts">
	import logger from '$lib/logger';
	import { panelAuthState, type PanelAuthState } from '$lib/panelAuthState';
	import { panelState } from '$lib/panelState';
	import { goto as gotoOnce } from '$app/navigation';
	import { page } from '$app/stores';
	import Loading from './Loading.svelte';
	import { panelQuery } from '$lib/fetch';
	import ErrorComponent from './Error.svelte';
	import { panelHelloProtocolVersion } from '$lib/constants';
	import type { Hello } from '$lib/generated/arcadia/Hello';

	let loadingMsg = 'Waiting for monkeys?';
	let navigating: boolean = false;

	// Safari needs this patch here
	const goto = async (url: string) => {
		if (navigating) return new Promise(() => {});
		navigating = true;
		return await gotoOnce(url);
	};

	const setupState = async () => {
		if ($panelState) {
			return true;
		}

		logger.info('Panel', 'Page:', { $page });

		let authorized = false;

		logger.info('Panel', 'Loading panel...');

		loadingMsg = 'Checking authentication';

		let panelStateData = localStorage.getItem('panelStateData');

		if (panelStateData) {
			try {
				let json: PanelAuthState = JSON.parse(panelStateData);
				$panelAuthState = json;

				switch ($panelAuthState?.sessionState) {
					case 'pending':
						await goto(`/login/mfa?redirect=${window.location.pathname}`);
						return false;
				}

				authorized = true;
			} catch (e) {
				logger.error('Panel', 'Failed to load panel state data from localStorage');

				if ($page?.url?.pathname != '/login') {
					await goto(`/login?redirect=${window?.location?.pathname}`);
				}
				return false;
			}
		}

		if (!authorized) {
			if ($page.url.pathname != '/login') {
				await goto(`/login?redirect=${window.location.pathname}`);
			}
			return false;
		}

		loadingMsg = 'Validating your existence...';

		let helloRes = await panelQuery({
			Hello: {
				version: panelHelloProtocolVersion,
				login_token: $panelAuthState?.loginToken || ''
			}
		});

		if (!helloRes.ok) {
			let err = await helloRes.text();
			throw new Error(err?.toString() || 'Unknown error');
		}

		let helloData: Hello = await helloRes.json();

		$panelState = helloData;

		setInterval(updateAuthState, 1000 * 30);

		return true;
	};

	const updateAuthState = async () => {
		logger.info('Panel.CheckAuth', 'Checking auth...');

		try {
			let helloRes = await panelQuery({
				Hello: {
					version: panelHelloProtocolVersion,
					login_token: $panelAuthState?.loginToken || ''
				}
			});

			if (!helloRes.ok) {
				let err = await helloRes.text();
				if ($panelAuthState) {
					$panelAuthState.authErr = 'hello_failed';
				}
				throw new Error(err?.toString() || 'Unknown error');
			}

			let helloData: Hello = await helloRes.json();

			$panelState = helloData;
		} catch (err) {
			logger.error('Panel.CheckAuth', err);
		}
	};
</script>

{#await setupState()}
	<Loading msg={loadingMsg} />
{:then res}
	{#if res}
		{#if $panelAuthState?.authErr}
			<p class="text-xl text-red-400">Authentication failed: {$panelAuthState?.authErr}</p>
		{/if}
		<slot />
	{:else}
		<Loading msg={'Just a moment...'} />
	{/if}
{:catch err}
	<ErrorComponent msg={err?.toString()} />
{/await}
