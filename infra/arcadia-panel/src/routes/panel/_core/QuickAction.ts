export interface Option {
	name: string;
	description: string;
	link: string;
	enabled: () => boolean;
}

export interface QuickAction {
	name: string;
	description: string;
	link: string;
	options?: () => Option[];
	enabled: () => boolean;
}
