import type { Capability, Field, FieldRenderMethod, FieldType } from './types';

interface NewFieldExtraOptions<T> {
	arrayLabel?: string;
	type?: FieldType;
	renderMethod?: FieldRenderMethod;
	customRenderer?: (cap: Capability, data: T) => Promise<string>;
	required?: boolean;
	disabled?: boolean;
}

export const newField = <T>(
	id: string,
	label: string,
	helpText: string,
	required: boolean,
	disabled: boolean,
	opts?: NewFieldExtraOptions<T>
): Field<T> => {
	return {
		id,
		label,
		helpText,
		type: opts?.type ? opts?.type : 'text',
		renderMethod: opts?.renderMethod ? opts?.renderMethod : 'text',
		customRenderer: opts?.customRenderer,
		required,
		disabled
	};
};
