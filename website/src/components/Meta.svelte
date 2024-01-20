<script lang="ts">
	export let title: string;
	export let description: string;
	export let image: string = '/logo.webp';
</script>

<svelte:head>
	<title>{title}</title>
	<meta property="og:title" content={title} />
	<meta property="og:site_name" content="AntiRaid" />
	<meta property="og:type" content="website" />
	<meta property="og:url" content="https://antiraid.xyz" />
	<meta property="og:image" content={image} />
	<meta property="og:description" content={description} />
</svelte:head>
