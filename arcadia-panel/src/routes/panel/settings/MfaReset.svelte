<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { logoutUser } from '$lib/logout';
	import { panelAuthState } from '$lib/panelAuthState';
	import { error, success } from '$lib/toast';
	import ButtonReact from '../../../components/button/ButtonReact.svelte';
	import GreyText from '../../../components/GreyText.svelte';
	import InputText from '../../../components/inputs/InputText.svelte';
	import { Color } from '../../../components/button/colors';
	import { panelAuthProtocolVersion } from '$lib/constants';

	let mfaOtp: string = '';

	const resetMfa = async () => {
		if (mfaOtp?.length != 6) {
			error('Please enter a valid 6-digit OTP');
			return false;
		}

		try {
			let res = await panelQuery({
				Authorize: {
					version: panelAuthProtocolVersion,
					action: {
						ResetMfaTotp: {
							login_token: $panelAuthState?.loginToken || '',
							otp: mfaOtp
						}
					}
				}
			});

			if (!res.ok) {
				error((await res.text()) || 'Failed to reset MFA');
				return false;
			}

			success('MFA reset successfully');
			logoutUser(false);
			return true;
		} catch (e) {
			error('Failed to reset MFA');
			return false;
		}
	};
</script>

<h1 class="mt-3 text-xl font-semibold">Reset MFA</h1>

<GreyText
	>Reset Panel 2FA here. Note that you will need access to your current OTP in order to continue.</GreyText
>

<div class="p-2" />

<InputText
	id="otp"
	minlength={6}
	label="Enter OTP"
	description="Please open your authenticator app and enter the <span class='font-bold'>One-Time Password</span> you have recieved now!"
	placeholder="Code"
	bind:value={mfaOtp}
/>

<ButtonReact
	color={Color.Themable}
	icon={'mdi:key'}
	text={'Verify OTP & Reset MFA'}
	states={{
		loading: 'Resetting MFA...',
		success: 'Successfully reset MFA!',
		error: 'Failed to reset MFA'
	}}
	onClick={resetMfa}
/>
