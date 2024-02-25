<script lang="ts">
	import { panelState } from '$lib/panelState';
</script>

<div class="mt-14">
	<h1 class="text-3xl font-semibold">
		Welcome Back, {$panelState?.staff_member?.user?.display_name}!
	</h1>

	<p>Pretty empty right now, isn't it?</p>
</div>
