<script lang="ts">
	import logger from '$lib/logger';
	import { panelState } from '$lib/panelState';
	import { build, hasPerm } from '$lib/perms';
	import { title } from '$lib/strings';
	import AuthBoundary from '../../components/AuthBoundary.svelte';
	import ListItem from '../../components/ListItem.svelte';
	import UnorderedList from '../../components/UnorderedList.svelte';
	import InfoPane from '../../components/pane/InfoPane.svelte';
	import PaneContent from '../../components/pane/PaneContent.svelte';
	import PaneWrapper from '../../components/pane/PaneWrapper.svelte';
	import PermDisplay from './_core/PermDisplay.svelte';
	import type { QuickAction } from './_core/QuickAction';
	import QuickMenuOption from './_core/QuickMenuOption.svelte';
	import { staffQuickActions } from './staff/quickActions';

	let quickActions: QuickAction[] = [
		{
			name: 'Index',
			description: 'Index Page',
			link: '/panel',
			enabled: () => {
				logger.info('QuickAction', $panelState);
				return true;
			}
		},
		{
			name: 'Staff Guide',
			description: 'View our staff guides for guidance on doing stuff!',
			link: '/panel/staffguide',
			enabled: () => {
				logger.info('QuickAction', $panelState);
				return true;
			}
		},
		{
			name: 'Bot Queue',
			description: 'View the bot queue',
			link: '/panel/bots/queue',
			enabled: () => true // This is always available
		},
		{
			name: 'List Management',
			description: 'Manage the list',
			link: '',
			enabled: () => true,
			options: () => {
				return [
					{
						name: 'CDN',
						description: 'Manage the CDN(s) modifiable by this Arcadia instance',
						link: '/panel/cdn',
						enabled: () =>
							hasPerm($panelState?.staff_member?.resolved_perms || [], build('cdn', 'list_scopes'))
					},
					{
						name: 'Partners',
						description: 'View and/or manage the partners on the list',
						link: '/panel/partners',
						enabled: () => true // All staff can view the partner list, other permissions are handled by admin panel code
					},
					{
						name: 'Changelogs',
						description: 'View and/or manage the changelogs for the list',
						link: '/panel/changelogs',
						enabled: () => true // All staff can view the changelog entry list, other permissions are handled by admin panel code
					},
					{
						name: 'Blog',
						description: 'Manage the blog posts for the list',
						link: '/panel/blog',
						enabled: () => true // All staff can view the blog post list, other permissions are handled by admin panel code
					},
					{
						name: 'Applications',
						description: 'Manage the applications for the list',
						link: '/panel/apps',
						enabled: () =>
							hasPerm($panelState?.staff_member?.resolved_perms || [], build('apps', 'view'))
					}
				];
			}
		},
		{
			name: 'RPC Actions',
			description: 'Manage entities!',
			link: '/panel/rpc',
			enabled: () => true,
			options: () =>
				($panelState?.target_types || []).map((type) => {
					return {
						name: type,
						description: `Manage ${type}s!`,
						link: `/panel/rpc/${type}`,
						enabled: () => true
					};
				})
		},
		{
			name: 'Staff Management',
			description: 'View and manage staff',
			enabled: () => true,
			link: '/panel/staff',
			options: () => {
				return staffQuickActions;
			}
		},
		{
			name: 'Settings',
			description: 'Customize your experience!',
			link: '/panel/settings',
			enabled: () => true
		},
		{
			name: 'Logout',
			description: 'Logout from the panel',
			link: '/panel/logout',
			enabled: () => true
		}
	]; // cum
</script>

<AuthBoundary>
	<PaneWrapper>
		<InfoPane title="Navigation" description="Welcome to the panel">
			<div>
				{#each quickActions as action, index}
					{#if action.enabled()}
						<QuickMenuOption {index} {action} actionsLength={quickActions.length} />
					{/if}
				{/each}
			</div>

			<div class="mt-4" />

			<details>
				<summary class="hover:cursor-pointer">View Permissions</summary>

				<div class="p-2" />

				<span class="font-semibold">Staff Positions:</span>
				<UnorderedList>
					{#each $panelState?.staff_member?.positions || [] as staffPosition}
						<ListItem className="ml-3"
							>{title(staffPosition.name.replaceAll('_', ' '))}
							<span class="opacity-80">({staffPosition.name})</span></ListItem
						>
					{/each}
				</UnorderedList>

				{#if ($panelState?.staff_member?.perm_overrides || []).length > 0}
					<span class="font-semibold">Permission Overrides:</span>
					<UnorderedList>
						{#each $panelState?.staff_member?.perm_overrides || [] as perm}
							<PermDisplay {perm} />
						{/each}
					</UnorderedList>
				{/if}

				<span class="font-semibold">Resolved Permissions:</span>
				<UnorderedList>
					{#each $panelState?.staff_member?.resolved_perms || [] as perm}
						<PermDisplay {perm} />
					{/each}
				</UnorderedList>
			</details>
		</InfoPane>

		<PaneContent>
			<div class="block mt-14">
				<p>
					{$panelState?.instance_config?.description}
				</p>
				{#if $panelState?.instance_config?.warnings}
					<div class="text-yellow-500 rounded-lg">
						{#each $panelState.instance_config?.warnings as warning}
							<p>{warning}</p>
						{/each}
					</div>
				{/if}
				<hr class="my-4" />
				<slot />
			</div>
		</PaneContent>
	</PaneWrapper>
</AuthBoundary>
