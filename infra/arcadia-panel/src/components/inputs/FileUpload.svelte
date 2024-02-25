<script lang="ts">
	import logger from '$lib/logger';
	import { error } from '$lib/toast';
	import Label from './Label.svelte';

	// Needed props
	export let acceptableTypes: string[];
	export let id: string;
	export let label: string;

	// Can be bound to
	export let file: File;
	export let fileName: string = '';
	export let fileMimeType: string = '';
	export let fileUploaded: boolean = false;

	let fileList: FileList;
	const readFile = () => {
		logger.info('FileUpload', 'Reading file');
		fileUploaded = false;

		if (fileList.length > 1) {
			error('Please only upload one file');
			return;
		}

		let fileTmp = fileList[0];

		if (acceptableTypes.length > 0 && !acceptableTypes.includes(fileTmp.type)) {
			error(`Please upload an ${acceptableTypes}`);
			return;
		}

		fileMimeType = fileTmp.type;
		fileName = fileTmp.name;
		file = fileTmp;

		fileUploaded = true;
	};

	$: if (fileList) {
		readFile();
	}
</script>

<div class="file-upload mb-3">
	<Label {id} {label} />
	<br />
	<input
		accept={acceptableTypes.length > 0 ? acceptableTypes.join(', ') : null}
		bind:files={fileList}
		on:change={() => readFile()}
		{id}
		name={id}
		type="file"
		multiple={false}
	/>
</div>
