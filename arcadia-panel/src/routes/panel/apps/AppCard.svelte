<script lang="ts">
	import type { AppResponse } from '$lib/comp_types/apps';
	import { fetchClient, panelQuery } from '$lib/fetch';
	import type { Query } from '$lib/generated/htmlsanitize/Query';
	import { panelAuthState } from '$lib/panelAuthState';
	import { panelState } from '$lib/panelState';
	import { build, hasPerm } from '$lib/perms';
	import { title } from '$lib/strings';
	import { error, success } from '$lib/toast';
	import Card from '../../../components/Card.svelte';
	import CardButton from '../../../components/CardButton.svelte';
	import Modal from '../../../components/Modal.svelte';
	import ObjectRender from '../../../components/ObjectRender.svelte';
	import ButtonReact from '../../../components/button/ButtonReact.svelte';
	import { Color } from '../../../components/button/colors';
	import InputTextArea from '../../../components/inputs/InputTextArea.svelte';
	import Select from '../../../components/inputs/select/Select.svelte';

	export let app: AppResponse;
	export let index: number;

	let showHtmlForQuestions: { [key: string]: string } = {};
	let htmlButtonText: { [key: string]: string } = {};
	let showActionsModal: boolean = false;
	let actionApproveDenyApp: string = '';
	let actionApproveDenyFeedback: string = '';

	const approveDenyAction = async () => {
		if (!actionApproveDenyApp) {
			error('Please select an action');
			return false;
		}

		let res = await panelQuery({
			PopplioStaff: {
				login_token: $panelAuthState?.loginToken || '',
				path: `/staff/apps/${app?.app_id}`,
				method: 'PATCH',
				body: JSON.stringify({
					approved: actionApproveDenyApp == 'approve',
					reason: actionApproveDenyFeedback
				})
			}
		});

		if (!res.ok) {
			let err = await res.json();

			throw new Error(
				`Failed to approve application: ${err?.message?.toString() || err || 'Unknown error'}`
			);
		}

		success('Application approved!');
		return true;
	};

	const showAsHtml = async (id: string, answer: string) => {
		if (showHtmlForQuestions[id]) {
			delete showHtmlForQuestions[id];
			delete htmlButtonText[id];
			showHtmlForQuestions = showHtmlForQuestions;
			htmlButtonText = htmlButtonText;
			return true;
		}

		htmlButtonText[id] = 'Sanitizing...';

		let query: Query = {
			SanitizeRaw: {
				body: answer
			}
		};

		let res = await fetchClient(`${$panelState?.core_constants?.htmlsanitize_url}/query`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(query)
		});

		if (!res.ok) {
			let err = await res.json();

			htmlButtonText[id] =
				'Failed to sanitize: ' +
				(err?.message?.toString() || err || 'Unknown error') +
				'. Click to try again';

			return false;
		}

		let sanitized = await res.text();

		showHtmlForQuestions[id] = sanitized;

		htmlButtonText[id] = 'Hide HTML';

		return true;
	};
</script>

<Card>
	<img slot="image" src={app?.user?.avatar} alt="" />
	<svelte:fragment slot="display-name"
		>{app?.user?.display_name} ({app?.user?.username})</svelte:fragment
	>
	<svelte:fragment slot="short">
		<p class="font-semibold">Position: {title(app?.position)}</p>
		<p class="font-semibold">App ID: {title(app?.app_id)}</p>
		<div class="mb-4"></div>
		{#each app?.questions as question, i}
			<div class="mb-3">
				<p class="font-semibold"><em>Question {i + 1}</em></p>
				<p><strong>{question?.question}</strong></p>
				{#if question?.id in showHtmlForQuestions}
					<div class="desc prose text-white">
						{@html showHtmlForQuestions[question?.id]}
					</div>
				{:else}
					<p class="desc">{app?.answers[question?.id]}</p>
				{/if}

				<button
					class="text-themable-400 hover:text-themable-500"
					on:click={(e) => {
						e.preventDefault();
						showAsHtml(question?.id, app?.answers[question?.id]);
					}}
				>
					{htmlButtonText[question.id] || 'Show as HTML'}
				</button>

				<details>
					<summary class="hover:cursor-pointer">Question Data</summary>
					<ObjectRender object={question} />
				</details>
			</div>
		{/each}
	</svelte:fragment>
	<svelte:fragment slot="index">#{index + 1}</svelte:fragment>
	<svelte:fragment slot="type">
		{title(app?.position)} [{title(app?.state)}]
	</svelte:fragment>
	<svelte:fragment slot="actionA">
		{#if app?.state == 'pending' && hasPerm($panelState?.staff_member?.resolved_perms || [], build('apps', 'approve_deny'))}
			<CardButton icon="mdi:edit" onClick={() => (showActionsModal = true)}>
				Approve/Deny
			</CardButton>
		{/if}
	</svelte:fragment>
</Card>

{#if showActionsModal}
	<Modal bind:showModal={showActionsModal}>
		<h1 slot="header" class="font-semibold text-2xl">Approve/Deny Application</h1>

		<p class="break-all"><strong>App ID:</strong> {app?.app_id}</p>
		<p class="break-all"><strong>Position:</strong> {app?.position}</p>
		<p class="break-all"><strong>App State:</strong> {app?.state}</p>
		<p class="break-all">
			<strong>App User:</strong>
			{app?.user?.username} [{app?.user?.display_name}]
		</p>

		<InputTextArea
			id="actionApproveDenyFeedback"
			label="Feedback"
			placeholder="Feedback for the user's application"
			minlength={0}
			showErrors={false}
			bind:value={actionApproveDenyFeedback}
		/>

		<Select
			bind:value={actionApproveDenyApp}
			id="actionApproveDenyApp"
			label="Action"
			choices={[
				{
					id: 'approve',
					value: 'approve',
					label: 'Approve'
				},
				{
					id: 'deny',
					value: 'deny',
					label: 'Deny'
				}
			]}
		/>

		{#if hasPerm($panelState?.staff_member?.resolved_perms || [], build('apps', 'approve_deny'))}
			<ButtonReact
				color={Color.Themable}
				text={actionApproveDenyApp == 'approve' ? 'Approve' : 'Deny'}
				icon="mdi:check"
				states={{
					loading: actionApproveDenyApp == 'approve' ? 'Approving...' : 'Denying...',
					success: actionApproveDenyApp == 'approve' ? 'Approved!' : 'Denied!',
					error: 'Failed to approve/deny'
				}}
				onClick={approveDenyAction}
			/>
		{/if}
	</Modal>
{/if}
