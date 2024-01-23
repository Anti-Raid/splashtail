<script lang="ts">
	import Loading from '../../../components/Loading.svelte';
	import ErrorComponent from '../../../components/Error.svelte';
	import type { Query } from '$lib/generated/htmlsanitize/Query';
	import { panelState } from '$lib/panelState';

	const fetchGuide = async () => {
		const guideFile = await fetch(
			`${$panelState?.core_constants?.cdn_url}/staff/guide.md?n=${Date.now()}`
		);

		if (!guideFile.ok) throw new Error('Failed to fetch guide');

		const guideText = await guideFile.text();

		// HTMLSanitize it
		let hsq: Query = {
			SanitizeRaw: {
				body: guideText
			}
		};
		const guideHtml = await fetch(`${$panelState?.core_constants?.htmlsanitize_url}/query`, {
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
</script>

{#await fetchGuide()}
	<Loading msg="Fetching guide..." />
{:then resp}
	<div class="px-3 desc" id="text">
		{@html resp.text}
	</div>
{:catch error}
	<ErrorComponent msg={`Something went wrong: ${error.message}`} />
{/await}
