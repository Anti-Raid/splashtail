<script lang="ts">
	import Loading from '../../../components/Loading.svelte';
	import ErrorComponent from '../../../components/Error.svelte';
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState, type PanelAuthState } from '$lib/panelAuthState';
	import { goto as gotoOnce } from '$app/navigation';
	import { hexToUtf8 } from '$lib/strings';
	import logger from '$lib/logger';
	import { panelAuthProtocolVersion } from '$lib/constants';
	import type { ILoginState } from '$lib/iloginState';

	// Safari needs this patch here
	let navigating: boolean = false;
	const goto = async (url: string) => {
		if (navigating) return new Promise(() => {});
		navigating = true;
		return await gotoOnce(url);
	};

	let msg: string = 'Logging you in now...';

	const authorize = async () => {
		let searchParams = new URLSearchParams(window.location.search);

		let code = searchParams.get('code');

		if (!code) {
			throw new Error('Failed to get code from URL');
		}

		let state = searchParams.get('state');

		if (!state) {
			throw new Error('Failed to get state from URL');
		}

		let loginState: ILoginState = JSON.parse(hexToUtf8(state));

		if (!loginState) {
			throw new Error('Failed to parse login state');
		}

		$panelAuthState = {
			url: loginState?.instanceUrl,
			loginToken: '',
			sessionState: 'noSession'
		};

		let res = await panelQuery({
			Authorize: {
				version: panelAuthProtocolVersion,
				action: {
					CreateSession: {
						code: code,
						redirect_url: `${window.location.origin}/login/authorize`
					}
				}
			}
		});

		if (!res.ok) {
			throw new Error((await res.text()) || 'Failed to login');
		}

		let loginToken = await res.text();

		let ps: PanelAuthState = {
			url: loginState?.instanceUrl,
			loginToken: loginToken,
			sessionState: 'pending'
		};

		localStorage.setItem('panelStateData', JSON.stringify(ps));

		if (!localStorage.getItem('panelStateData')) {
			throw new Error('Failed to save panel state data to localStorage');
		}

		logger.info('Panel', 'Login', localStorage.getItem('panelStateData'));

		if (window.opener) {
			window?.opener?.postMessage('login', location.origin);
		} else {
			return await goto(`/login/mfa?redirect=${loginState?.redirectUrl}`);
		}
	};
</script>

{#await authorize()}
	<Loading {msg} />
{:then}
	<Loading msg="Just one moment..." />
{:catch err}
	<ErrorComponent msg={err?.toString() || 'Unknown error occurred'} />
{/await}
