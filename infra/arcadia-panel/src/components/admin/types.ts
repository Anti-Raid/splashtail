import type { Color } from '../button/colors';
import type { ButtonStatesWithNormal } from '../button/states';
import type { SMOption } from '../inputs/select/select';
import type { WarningBox } from '../warningbox/warningBox';

export type FieldType =
	| 'text'
	| 'textarea' // Long (extended answer)
	| 'text[]' // Array of text
	| 'text[kv]' // Key-value pair of text to text
	| 'ibl:link' // Extra links (IBL specific type)
	| 'text[choice]' // Select menu for text
	| 'number'
	| 'boolean'
	| 'file';

/**
 * How the field is rendered in the data table
 *
 * Defaults: text as default, unordered-list for array
 */
export type FieldRenderMethod =
	| 'text'
	| 'unordered-list'
	| 'ordered-list'
	| 'custom'
	| 'custom[html]'
	| 'none';

export type Capability = 'view' | 'create' | 'update' | 'delete';

// Field fetch single represents a single field fetch
export type FieldFetchSingle<T> =
	| ((cap: Capability, reason?: string) => Promise<Field<T> | null>)
	| Field<T>
	| null;
export type FieldFetch<T> = FieldFetchSingle<T>[];

// Custom action types
export type CustomActionFetchSingle<T> =
	| ((cap: Capability, reason?: string) => Promise<CustomAction<T>>)
	| CustomAction<T>
	| null;
export type CustomActionFetch<T> = CustomActionFetchSingle<T>[];

/**
 * Data for a file upload field
 */
export interface FieldFileUploadData {
	/**
	 * Acceptable mime types for the file upload
	 */
	acceptableMimeTypes: string[];
	/**
	 * A function to render a preview, if null a preview won't be rendered
	 */
	renderPreview: (cap: Capability, file: File, box: HTMLDivElement) => Promise<void>;
}

/**
 * Data for a field
 */
export interface Field<T> {
	/**
	 * The id of the field
	 */
	id: string;
	/**
	 * The label of the field
	 */
	label: string;
	/**
	 * The 'array' label of the field. Is optional
	 */
	arrayLabel?: string;
	/**
	 * The type of the field
	 */
	type: FieldType;
	/**
	 * Placeholder/help text of the field
	 */
	helpText: string;
	/**
	 * Whether or not the field is required
	 *
	 * This flag is frontend only
	 */
	required: boolean;
	/**
	 * Whether or not the field is disabled
	 *
	 * This flag is frontend only
	 */
	disabled: boolean;
	/**
	 * If this is a file upload, this must be set
	 */
	fileUploadData?: FieldFileUploadData;
	/**
	 * Select menu choices (if it is to be a choice field)
	 */
	selectMenuChoices?: SMOption[];
	/**
	 * Render method of the field
	 *
	 * Set to 'text' when in doubt
	 */
	renderMethod: FieldRenderMethod;
	/**
	 * Custom renderer function. Note that renderMethod must be custom when this is set
	 */
	customRenderer?: (cap: Capability, data: T) => Promise<string>;
}

/**
 * A custom action is an action that can be performed by clicking a button
 */
export interface CustomAction<T> {
	/**
	 * The label of the action
	 */
	label: string;
	/**
	 * The help text of the action
	 */
	helpText: string;
	/**
	 * The action to call when the button is clicked
	 */
	action: (cap: Capability, data: T, div: HTMLDivElement) => Promise<boolean>;
	/**
	 * A warning box (optional) for the action
	 */
	warningBox?: (
		cap: Capability,
		data: T,
		div: HTMLDivElement,
		func: () => Promise<boolean>
	) => WarningBox;
	/**
	 * Button configuration
	 */
	button: {
		/**
		 * Button icon
		 */
		icon: string;
		/**
		 * Button color
		 */
		color: Color;
		/**
		 * Button states
		 */
		states: ButtonStatesWithNormal;
	};
}

/**
 * This contains the data for a create/upload/delete that will be sent
 */
export interface Entry<T> {
	/**
	 * Files being created/updated
	 */
	files: { [key: string]: File };
	/**
	 * Data being created/updated
	 */
	data: T;
	/**
	 * Function that when called adds a status entry
	 */
	addStatus: (s: string) => void;
}

export interface BaseSchema<T> {
	/**
	 * The name of the schema
	 */
	name: string;
	/**
	 * The fields of the schema
	 */
	fields: FieldFetch<T>;
	/**
	 * Strictly verify that data has same keys as schema
	 */
	strictSchemaValidation: boolean;
	/**
	 * Fields to ignore for schema validation
	 */
	strictSchemaValidationIgnore: string[];
	/**
	 * Returns the capabilities the user has regarding the schema
	 *
	 * @returns The capabilities of the schema
	 */
	getCaps(): Capability[];
	/**
	 * Returns the primary key of the schema
	 *
	 * @param cap The capability that is being exercised
	 * @returns The id/name of the primary key
	 */
	getPrimaryKey: (cap: Capability) => string;
	/**
	 * The function to fetch all data currently present in the database
	 *
	 * @returns A list of data entries corresponding to the schema
	 */
	viewAll: () => Promise<T[]>;
	/**
	 * The function to fetch a specific row by pkey value
	 *
	 * @return A data entry
	 */
	view: (key: string, value: string) => Promise<T | null | undefined>;
	/**
	 * A function to create a new data entry in the database
	 *
	 * @param data The data to add
	 * @returns Whether the data was added successfully
	 */
	create: (data: Entry<T>) => Promise<void>;
	/**
	 * A function to update an existing data entry in the database
	 *
	 * @param data The data to update
	 * @returns Whether the data was updated successfully
	 */
	update: (data: Entry<T>) => Promise<void>;
	/**
	 * A function to delete an existing data entry in the database
	 *
	 * @param data The data to delete
	 * @returns Whether the data was deleted successfully
	 */
	delete: (data: Entry<T>) => Promise<void>;
}

/**
 * Wrapper type to allow viewToTable to also return the fields
 */
export interface ViewToTableResponse<T> {
	fields: FieldFetch<T>;
	data: any[];
}

/**
 * Schema is the data that the admin panel will use to render
 *
 * T is the type of the schema and V is the type of the data for the purpose of tabular display
 */
export interface Schema<T> extends BaseSchema<T> {
	/**
	 * This function takes in viewed data and turns it into DataTable rows (which can be of any list type)
	 *
	 * @param data The data to convert
	 * @returns The converted data
	 */
	viewToTable: (data: T[]) => Promise<ViewToTableResponse<T>>;
	/**
	 * This function takes in a capability and responds with a warningbox
	 *
	 * Note that only 'delete' actually supports warnings at this time
	 */
	warningBox: (cap: Capability, data: T, func: () => Promise<boolean>) => WarningBox;
	/**
	 * This function is called when a modal is opened
	 */
	onOpen: (cap: Capability, evt: string, data?: T) => any;
	/**
	 * List of custom actions
	 */
	customActions?: CustomActionFetch<T>;
}

export interface ManageSchema<T> {
	schema: Schema<T>;
	manageData: T;
}
