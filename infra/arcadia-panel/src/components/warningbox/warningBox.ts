import type { ButtonStatesWithNormal } from '../button/states';

export interface WarningBox {
	header: string;
	text: string;
	nonce?: string;
	buttonStates: ButtonStatesWithNormal;
	inputtedText?: string;
	onConfirm: () => Promise<boolean>;
}

export const setupWarning = (wb: WarningBox) => {
	wb.nonce =
		Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
	wb = wb; // Ensure warningBox is updated
};

export const commonButtonReactStates = {
	loading: 'Please Wait...',
	success: 'Showing confirmation box now...',
	error: 'Failed to show confirmation box'
};
