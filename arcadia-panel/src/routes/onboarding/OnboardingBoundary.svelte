<script lang="ts">
	import { utf8ToHex } from '$lib/strings';
	import Loading from '../../components/Loading.svelte';
	import { obBoundary } from './obBoundaryState';
	import ErrorComponent from '../../components/Error.svelte';
	import { persepolisUrl } from './onboardingConsts';
	import type { AuthData } from '$lib/generated/persepolis/AuthData';
	import logger from '$lib/logger';

	const login = () => {
		localStorage?.setItem('obCurrentUrl', window?.location?.toString());

		let finalPath = utf8ToHex(`${window?.location?.origin}/onboarding/authorize`);

		// Redirect to the login page
		window.location.href = `${persepolisUrl}/create-login?state=create_session.${finalPath}`;
	};

	const checkToken = async () => {
		logger.info('OBBoundary', 'Checking token');

		let searchParams = new URLSearchParams(window.location.search);

		if (searchParams.get('token')) {
			let token = searchParams.get('token');

			localStorage.setItem(
				'obBoundary',
				JSON.stringify({
					token: token
				})
			);

			window.location.href = localStorage.getItem('obCurrentUrl') || '/';

			return;
		}

		if (localStorage?.getItem('obBoundary')) {
			let obBoundaryData = JSON.parse(localStorage.getItem('obBoundary') || '{}');

			let res = await fetch(`${persepolisUrl}/auth-data`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					login_token: obBoundaryData?.token
				})
			});

			if (!res.ok) {
				// Invalid token
				login();
				throw new Error('Invalid token, logging you in');
			}

			let authData: AuthData = await res.json();

			$obBoundary = {
				token: obBoundaryData?.token,
				authData
			};
			return;
		}

		// No token
		login();
		throw new Error('No token found, logging you in...');
	};
</script>

{#await checkToken()}
	<Loading msg="Checking token..." />
{:then _}
	<slot />
{:catch error}
	<ErrorComponent msg={error?.toString() || 'Unknown error'} />
{/await}
