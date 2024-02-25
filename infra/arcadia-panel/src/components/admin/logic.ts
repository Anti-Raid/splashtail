import type { Link } from '$lib/generated/arcadia/Link';
import logger from '$lib/logger';
import type {
	Field,
	FieldFetch,
	Capability,
	Schema,
	CustomActionFetch,
	CustomAction
} from './types';

export const fetchFields = async <T>(cap: Capability, ff: FieldFetch<T>, reason?: string) => {
	let fields: Field<T>[] = [];
	logger.info('FetchFields', cap, ff);
	for (let field of ff) {
		if (!field) {
			continue;
		}

		if (field instanceof Function) {
			let f = field as (cap: Capability, reason?: string) => Promise<Field<T> | null>;
			let fCap = await f(cap, reason);

			if (!fCap) {
				continue;
			}

			// File
			if (fCap?.fileUploadData && fCap?.type != 'file') {
				throw new Error('File upload field must have type file');
			}
			if (fCap?.type == 'file' && !fCap?.fileUploadData) {
				throw new Error('File upload field must have fileUploadData');
			}

			// Select menu
			if (fCap?.selectMenuChoices?.length && fCap?.type != 'text[choice]') {
				throw new Error('Select menu choices field must have type text[choice]');
			}
			if (fCap?.type == 'text[choice]' && !fCap?.selectMenuChoices) {
				throw new Error('Text choice field must have selectMenuChoices');
			}

			// Custom renderer
			const customRenderers = ['custom', 'custom[html]'];

			if (fCap?.customRenderer && !customRenderers.includes(fCap?.renderMethod)) {
				throw new Error('Custom renderer field must have renderMethod set to custom');
			}
			if (customRenderers.includes(fCap?.renderMethod) && !fCap?.customRenderer) {
				throw new Error('Custom render method field must have customRenderer set');
			}

			fields.push(fCap);
			continue;
		}
		fields.push(field as Field<T>);
	}

	return fields;
};

export const fetchCustomActions = async <T>(
	cap: Capability,
	fca: CustomActionFetch<T>,
	reason?: string
) => {
	let actions: CustomAction<T>[] = [];
	logger.info('FetchCustomActions', cap, fca);
	for (let action of fca) {
		if (!action) {
			continue;
		}

		if (action instanceof Function) {
			let f = action as (cap: Capability, reason?: string) => Promise<CustomAction<T> | null>;
			let fCap = await f(cap, reason);

			if (!fCap) {
				continue;
			}

			actions.push(fCap);
			continue;
		}
		actions.push(action as CustomAction<T>);
	}

	return actions;
};

export const validateDataWithFields = <T>(data: any, schema: Schema<T>, fields: Field<T>[]) => {
	let fieldSet = new Set(
		fields.filter((f) => !schema.strictSchemaValidationIgnore?.includes(f.id)).map((f) => f.id)
	);
	let dataKeySet = new Set(
		Object.keys(data).filter((k) => !schema.strictSchemaValidationIgnore?.includes(k))
	);

	// Get the difference between the field set and the data key set
	let fs_minus_dks = new Set([...fieldSet].filter((x) => !dataKeySet.has(x)));
	let dks_minus_fs = new Set([...dataKeySet].filter((x) => !fieldSet.has(x)));

	// If the schema is strict, then we should error if there are any keys in the data that are not in the schema
	if (fs_minus_dks.size > 0) {
		throw new Error(
			`Data contains keys that are not in the schema: ${Array.from(fs_minus_dks).join(', ')}`
		);
	}

	// If the schema is strict, then we should error if there are any keys in the schema that are not in the data
	if (dks_minus_fs.size > 0) {
		throw new Error(
			`Schema contains keys that are not in the data: ${Array.from(dks_minus_fs).join(', ')}`
		);
	}
};

export const cast = <T>(v: any): T => {
	return v as T;
};

export const castToArray = <T>(v: any): T[] => {
	return cast<any[]>(v);
};

export const castToLink = (v: any): Link => {
	return cast<Link>(v);
};
