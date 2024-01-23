<script lang="ts">
	import OrderedList from '../../components/OrderedList.svelte';
	import ListItem from '../../components/ListItem.svelte';
	import InputText from '../../components/inputs/InputText.svelte';
	import ButtonReact from '../../components/button/ButtonReact.svelte';
	import { error } from '$lib/toast';
	import { panelAuthState } from '$lib/panelAuthState';
	import { onMount } from 'svelte';
	import { goto as gotoOnce } from '$app/navigation';
	import { panelQuery } from '$lib/fetch';
	import logger from '$lib/logger';
	import { utf8ToHex } from '$lib/strings';
	import { Color } from '../../components/button/colors';
	import {
		panelAuthProtocolVersion,
		panelStartAuthRequestScope,
		panelStartAuthResponseScope
	} from '$lib/constants';
	import type { StartAuth } from '$lib/generated/arcadia/StartAuth';
	import type { ILoginState } from '$lib/iloginState';

	// Safari needs this patch here
	let navigating: boolean = false;
	const goto = async (url: string) => {
		if (navigating) return new Promise(() => {});
		navigating = true;
		return await gotoOnce(url);
	};

	onMount(async () => {
		if ($panelAuthState) {
			await goto('/');
		}
	});

	let instanceUrl = 'https://prod--panel-api.infinitybots.gg';

	const login = async () => {
		if (!instanceUrl) {
			error('Please enter an instance URL');
			return false;
		}

		if (
			!instanceUrl?.startsWith('https://') &&
			!instanceUrl?.startsWith('http://localhost') &&
			!instanceUrl?.startsWith('http://127.0.0.1')
		) {
			error('Instance URL must either be HTTPS or localhost');
			return false;
		}

		$panelAuthState = {
			url: instanceUrl,
			loginToken: '',
			sessionState: 'noSession'
		};

		let res = await panelQuery({
			Authorize: {
				version: panelAuthProtocolVersion,
				action: {
					Begin: {
						scope: panelStartAuthRequestScope,
						redirect_url: `${window.location.origin}/login/authorize`
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			error(err?.toString() || 'Unknown error');
			return false;
		}

		let loginData: StartAuth = await res.json();

		if (loginData?.scope != panelStartAuthRequestScope) {
			error('Invalid request scope. Are you using a compatible instance URL?');
			return false;
		}

		if (loginData?.response_scope != panelStartAuthResponseScope) {
			error('Invalid response scope. Are you using a compatible instance URL?');
			return false;
		}

		let redirectSearchParams = new URLSearchParams(window.location.search);
		let redirect = redirectSearchParams?.get('redirect');
		if (!redirect || !redirect.startsWith('/')) {
			redirect = '/';
		}

		let loginState: ILoginState = {
			instanceUrl,
			redirectUrl: redirect
		};

		let loginUrl = `${loginData?.login_url}&state=${utf8ToHex(JSON.stringify(loginState))}`;

		// Open login URL in new tab using window.open
		try {
			let loginTab = window.open(loginUrl, '_blank', 'popup');

			if (!loginTab) {
				throw new Error('No popups allowed');
			}

			loginTab?.focus();

			// Listen to message events
			window.addEventListener('message', async (e) => {
				// Check if the message is from the login tab
				if (e.source === loginTab) {
					loginTab?.close();
					await goto(`/login/mfa?redirect=${redirect}`);
				}
			});
		} catch (err) {
			logger.error('Popups seem to be disabled, falling back to redirect auth');

			// Open login URL in current tab using goto
			await goto(loginUrl);
		}

		return true;
	};
</script>

<article class="p-4">
	<h1 class="text-3xl font-semibold">Staff Login</h1>
	<p class="font-semibold text-lg">
		In order to login to the Arcadia instance, please input the 'Instance URL'.
		<br />
		<br />
		Note: The default instance URL is
		<a
			class="text-base font-semibold underline text-indigo-600 hover:text-indigo-400"
			href="https://prod--panel-api.infinitybots.gg">https://prod--panel-api.infinitybots.gg</a
		> and should be valid. If you wish to use a custom instance URL, please change the URL below.
	</p>
	<OrderedList>
		<ListItem>
			See #info in the staff server to check the status of panel if Login does not work
		</ListItem>
		<ListItem>Copy-paste any special 'Instance URL' given to you here</ListItem>
	</OrderedList>

	<hr class="my-4" />

	<InputText
		bind:value={instanceUrl}
		id="url"
		label="Instance URL"
		placeholder="https://prod--panel-api.infinitybots.gg"
		minlength={1}
		showErrors={false}
	/>

	<ButtonReact
		color={Color.Themable}
		icon={'mdi:login'}
		text={'Login'}
		states={{
			loading: 'Contacting instance...',
			success: 'Moving you along...',
			error: 'Failed to login'
		}}
		onClick={login}
	/>
</article>
