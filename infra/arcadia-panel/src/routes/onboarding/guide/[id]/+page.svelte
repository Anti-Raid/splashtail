<script lang="ts">
	import Loading from '../../../../components/Loading.svelte';
	import { cdnUrl, htmlSanitizeUrl, persepolisUrl } from '../../onboardingConsts';
	import ErrorComponent from '../../../../components/Error.svelte';
	import type { Query } from '$lib/generated/htmlsanitize/Query';
	import ButtonReact from '../../../../components/button/ButtonReact.svelte';
	import { Color } from '../../../../components/button/colors';
	import { fetchClient } from '$lib/fetch';
	import { obBoundary } from '../../obBoundaryState';
	import { page } from '$app/stores';
	import OnboardingBoundary from '../../OnboardingBoundary.svelte';

	const fetchGuide = async () => {
		const guideFile = await fetch(`${cdnUrl}/staff/guide.md?n=${Date.now()}`);

		if (!guideFile.ok) throw new Error('Failed to fetch guide');

		const guideText = await guideFile.text();

		// HTMLSanitize it
		let hsq: Query = {
			SanitizeRaw: {
				body: guideText
			}
		};
		const guideHtml = await fetch(`${htmlSanitizeUrl}/query`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(hsq)
		});

		if (!guideHtml.ok) throw new Error('Failed to sanitize guide');

		const guideHtmlText = await guideHtml.text();

		return {
			text: guideHtmlText
		};
	};

	const sha512 = async (str: string) => {
		const buf = await crypto.subtle.digest('SHA-512', new TextEncoder().encode(str));
		return Array.prototype.map
			.call(new Uint8Array(buf), (x) => ('00' + x.toString(16)).slice(-2))
			.join('');
	};

	let keyAdded: boolean = false;
	async function genSvu() {
		let verifyDat = await fetchClient(`${persepolisUrl}/onboarding-code`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				id: $page?.params?.id || '',
				login_token: $obBoundary?.token
			})
		});

		if (!verifyDat.ok) {
			let err = await verifyDat.text();
			throw new Error(err?.toString() || 'An unknown error occurred while loading the key');
		}

		let msg = await verifyDat.text();

		let uid = $obBoundary?.authData?.user_id?.toString().split('') || [];

		let key = msg.slice(-73).split('');

		// Some minor obfuscation
		key[2] = 'r';
		key[19] = uid[0];
		key[21] = uid[1];
		key[40] = uid[6];
		key[39] = 'x';
		let fkey = (await sha512(key.join(''))).slice(-6);

		return fkey;
	}

	async function addKeyToRandomLoc() {
		let key = null;

		try {
			key = await genSvu();
		} catch (e) {
			throw new Error(e?.toString() || 'An unknown error occurred while loading the key');
		}

		let flag = false;

		let text = document.querySelector('#text');

		if (!text) {
			throw new Error('No text element');
		}

		let possibleLocs = text.querySelectorAll('p, li');

		if (!possibleLocs || possibleLocs.length == 0) {
			return false;
		}

		// Randomly choose a paragraph that is not the last 2
		let randomLoc = Math.floor(Math.random() * (possibleLocs.length - 2));

		// Split it into words
		let words = possibleLocs[randomLoc].innerHTML.split(' ');

		let rand = 0;
		while (!flag) {
			rand = Math.floor(Math.random() * words.length);
			if (!words[rand] || words[rand].includes('https://')) {
				continue;
			}
			flag = true;
		}

		words[rand] += `. ${key}`;

		words[rand] = words[rand].replace('!', '').replace('.', '');

		possibleLocs[randomLoc].innerHTML = words.join(' ');

		keyAdded = true;
		return true;
	}
</script>

<OnboardingBoundary>
	{#await fetchGuide()}
		<Loading msg="Fetching guide..." />
	{:then resp}
		<div class="px-3 desc" id="text">
			{@html resp.text}
		</div>
		<div class="px-3 mb-2">
			{#if keyAdded}
				<p class="text-white mt-5 font-semibold">
					The staff verification code is somewhere in the guide.
					<br />
					Note that just trying to Ctrl-F it is not allowed and you may be demoted for lack of knowledge
					of the rules. Read the whole guide at least 5-10 times.
				</p>
			{:else}
				<p class="text-white mt-5 font-semibold">
					Be sure to read the entire staff guide before continuing. You will be demoted if you do
					not properly follow the rules
				</p>
				<ButtonReact
					color={Color.Themable}
					icon="fa-solid:code"
					states={{
						loading: 'Please wait...',
						success: 'Loaded code successfully!',
						error: 'Failed to load code'
					}}
					onClick={addKeyToRandomLoc}
					text="Show Code"
				/>
			{/if}
		</div>
	{:catch error}
		<ErrorComponent msg={`Something went wrong: ${error.message}`} />
	{/await}
</OnboardingBoundary>
