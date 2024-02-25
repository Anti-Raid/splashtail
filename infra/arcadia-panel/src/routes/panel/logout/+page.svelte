<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import Loading from '../../../components/Loading.svelte';
	import ErrorComponent from '../../../components/Error.svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { panelState } from '$lib/panelState';
	import { panelAuthProtocolVersion } from '$lib/constants';

	const clear = () => {
		localStorage.removeItem('panelStateData');

		$panelAuthState = {
			sessionState: 'loggedOut',
			loginToken: '',
			url: ''
		};

		$panelState = null;

		window.location.href = '/';
	};

	const logout = async () => {
		let res = await panelQuery({
			Authorize: {
				version: panelAuthProtocolVersion,
				action: {
					Logout: {
						login_token: $panelAuthState?.loginToken || ''
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			clear();
			throw new Error(`Failed to logout: ${err}`);
		}

		clear();
	};
</script>

{#await logout()}
	<Loading msg="Logging you out" />
{:then}
	<Loading msg="Redirecting you..." />
{:catch err}
	<ErrorComponent msg={`Logout failed: ${err?.toString()}`} />
{/await}
