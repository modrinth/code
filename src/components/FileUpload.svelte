<script lang="ts">
	import IconTrash from 'virtual:icons/heroicons-outline/trash'
	import IconUpload from 'virtual:icons/heroicons-outline/upload'
	import IconPhotograph from 'virtual:icons/heroicons-outline/photograph'
	import IconFile from 'virtual:icons/lucide/file'
	import { t } from 'svelte-intl-precompile'
	import Button from './Button.svelte'
	import { classCombine } from '../utils/classCombine'

	interface RemoteFile {
		name: string
		remote: true
	}

	export let multiple = false
	export let accept: string = '*'
	/** Prevents width from expanding due to large file names or images */
	export let constrained = false

	export let files: (File | RemoteFile)[] = []
	export let file: File | RemoteFile | undefined = undefined
	$: if (files) file = files[0] || undefined

	let inputElement: HTMLInputElement

	function addFiles(fileList: FileList) {
		for (const file of Array.from(fileList)) {
			// Check for duplicate files that aren't remote
			if (
				!files
					.filter((it) => !('remote' in file))
					.map((file) => file.name)
					.includes(file.name)
			) {
				if (files.map((file) => file.name).includes(file.name)) {
					// Remove remote file with the same name
					files = files.filter((it) => it.name !== file.name)
				}
				files = [...files, file]
			}
		}
	}
</script>

<div class={classCombine(['file-dropzone', constrained && 'file-dropzone--constrained'])}>
	{#if !file || multiple}
		<div
			class="file-dropzone__input"
			on:drop|preventDefault={(event) => addFiles(event.dataTransfer.files)}
			on:dragover|preventDefault
			on:click={() => {
				if (inputElement) inputElement.click()
			}}>
			<IconUpload />
			{$t('images.how_to')}

			<input
				type="file"
				{multiple}
				{accept}
				style:display="none"
				bind:this={inputElement}
				on:change={() => addFiles(inputElement.files)} />
		</div>
	{/if}

	{#each files as file (file.name)}
		<div class="file">
			<div class="file__tab">
				{#if !('remote' in file) && file.type.startsWith('image/')}
					<IconPhotograph />
				{:else}
					<IconFile />
				{/if}
				<div class="file__tab__name"><b>{file.name}</b></div>
				<Button
					color="tertiary"
					on:click={() => {
						files = files.filter((it) => it.name !== file.name)
					}}><IconTrash /> Remove</Button>
			</div>
			{#if !('remote' in file) && file.type.startsWith('image/')}
				<div class="file__preview">
					<img
						class="file__preview__image"
						src={URL.createObjectURL(file)}
						alt="Uploaded file preview" />
				</div>
			{/if}
		</div>
	{/each}
</div>

<style lang="postcss">
	.file-dropzone {
		display: flex;
		flex-direction: column;
		gap: 1rem;

		&--constrained {
			width: 27rem;
		}

		&__input {
			display: flex;
			padding: 1.5rem 2rem;
			justify-content: center;
			align-items: center;
			gap: 0.5rem;
			background-color: var(--color-button-bg);
			border-radius: var(--rounded-sm);
			border: dashed 0.3rem var(--color-text-lightest);
			cursor: pointer;
			color: var(--color-text-light);
		}
	}
</style>
