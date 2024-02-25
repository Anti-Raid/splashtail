<!--From https://svelte.dev/examples/modal -->
<script lang="ts">
	export let showModal: boolean; // boolean, whether or not the modal is shown or not

	let dialog: HTMLDialogElement; // HTMLDialogElement

	$: if (dialog && showModal) dialog.showModal();
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
<dialog
	class="px-4 py-2 rounded-md"
	bind:this={dialog}
	on:close={() => (showModal = false)}
	on:click|self={() => dialog.close()}
>
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<div on:click|stopPropagation>
		<!-- svelte-ignore a11y-autofocus -->
		<button class="close-btn font-semibold" autofocus on:click={() => dialog.close()}>Close</button>
		<slot name="header" />
		<hr />
		<div class="mb-4" />
		<slot />
		<div class="mb-4" />
	</div>
</dialog>

<style>
	dialog {
		width: 40em;
		border-radius: 0.2em;
		border: none;
		color: white;
		background: rgb(31, 31, 31);
	}
	dialog::backdrop {
		background: rgba(0, 0, 0, 0.2);
	}
	dialog > div {
		padding: 1em;
	}
	dialog[open] {
		animation: zoom 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
	}
	@keyframes zoom {
		from {
			transform: scale(0.95);
		}
		to {
			transform: scale(1);
		}
	}
	dialog[open]::backdrop {
		animation: fade 0.2s ease-out;
	}
	@keyframes fade {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.close-btn {
		margin-left: auto;
		float: right;
	}
</style>
