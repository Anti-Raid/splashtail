// From Infinity-Next commit 5317beadeeb039afe94f0a3027424c6322e64d40
//
// filename: src/components/layouts/Button.tsx
//
// Re-exported for panel use

export enum Color {
	Red = 'red',
	Orange = 'orange',
	Yellow = 'yellow',
	Green = 'green',
	Blue = 'blue',
	Themable = 'themable',
	Amber = 'amber',
	Violet = 'violet',
	Pink = 'pink',
	Emerald = 'emerald',
	Summer = 'summer'
}

export const getColor = (color: Color): [string, string] => {
	let colorClass = '';
	let iconClass = '';
	switch (color) {
		case Color.Red:
			colorClass = 'bg-red-600 hover:bg-red-800';
			iconClass = 'bg-red-700 text-amber-800';
			break;
		case Color.Themable:
			colorClass = 'bg-themable-600 hover:bg-themable-800';
			iconClass = 'bg-themable-800';
			break;
		case Color.Amber:
			colorClass = 'bg-amber-600 hover:bg-amber-800';
			iconClass = 'bg-amber-800';
			break;
		case Color.Orange:
			colorClass = 'bg-orange-600 hover:bg-orange-800';
			iconClass = 'bg-orange-800';
			break;
		case Color.Yellow:
			colorClass = 'bg-yellow-600 hover:bg-yellow-800';
			iconClass = 'bg-yellow-800';
			break;
		case Color.Green:
			// We need to be darker here for accessibility
			colorClass = 'bg-green-700 hover:bg-green-900';
			iconClass = 'bg-green-900';
			break;
		case Color.Blue:
			colorClass = 'bg-blue-600 hover:bg-blue-800';
			iconClass = 'bg-blue-800';
			break;
		case Color.Violet:
			colorClass = 'bg-violet-600 hover:bg-violet-800';
			iconClass = 'bg-violet-800';
			break;
		case Color.Pink:
			colorClass = 'bg-pink-600 hover:bg-pink-800';
			iconClass = 'bg-pink-800';
			break;
		case Color.Emerald:
			colorClass = 'bg-emerald-600 hover:bg-emerald-800';
			iconClass = 'bg-emerald-800';
			break;
		case Color.Summer:
			colorClass = 'bg-summer-600 hover:bg-emerald-800';
			iconClass = 'bg-summer-800';
			break;
		default:
			colorClass = 'bg-gray-600 hover:bg-gray-800';
			iconClass = 'bg-gray-800';
			break;
	}

	return [colorClass, iconClass];
};
