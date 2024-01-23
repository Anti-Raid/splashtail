import type { PlatformUser } from '$lib/generated/arcadia/PlatformUser';

export interface Question {
	id: string;
	question: string;
	paragraph: string;
	placeholder: string;
	short: boolean;
}
export interface Position {
	id: string;
	tags: string[];
	info: string;
	name: string;
	questions: Question[];
	hidden: boolean;
	closed: boolean;
}
export interface AppMeta {
	positions: Position[];
	stable: boolean; // Stable means that the list of apps is not pending big changes
}
export interface AppResponse {
	app_id: string;
	user_id: string;
	user: PlatformUser;
	questions: Question[];
	answers: { [key: string]: string };
	state: string;
	created_at: string /* RFC3339 */;
	position: string;
	review_feedback?: string;
}
export interface AppListResponse {
	apps: AppResponse[];
}
