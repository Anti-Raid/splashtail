import type { QuickAction } from '../_core/QuickAction';

export let staffQuickActions: QuickAction[] = [
	{
		name: 'Positions',
		description: 'View and/or manage staff positions',
		link: '/panel/staff/positions',
		enabled: () => true
	},
	{
		name: 'Members',
		description: 'View and/or manage staff members',
		link: '/panel/staff/members',
		enabled: () => true
	}
];
