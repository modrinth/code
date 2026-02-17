<template>
	<div class="flex w-full flex-col gap-4">
		<div class="font-semibold text-contrast">Upload custom modpack</div>

		<DropzoneFileInput
			v-if="!ctx.customModpackFile.value"
			primary-prompt="Drag and drop your .mrpack or .zip file"
			secondary-prompt="Or click to browse"
			accept=".zip,.mrpack"
			size="medium"
			@change="handleFileUpload"
		/>

		<div
			v-if="ctx.customModpackFile.value"
			class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
		>
			<div class="flex items-center gap-2 overflow-hidden">
				<FileIcon />
				<span
					v-tooltip="ctx.customModpackFile.value.name"
					class="overflow-hidden text-ellipsis whitespace-nowrap font-medium"
				>
					{{ ctx.customModpackFile.value.name }}
				</span>
			</div>

			<ButtonStyled size="standard" :circular="true">
				<button
					v-tooltip="'Replace file'"
					aria-label="Replace file"
					class="!shadow-none"
					@click="fileInput?.click()"
				>
					<ArrowLeftRightIcon aria-hidden="true" />
					<input
						ref="fileInput"
						class="hidden"
						type="file"
						accept=".zip,.mrpack"
						@change="handleFileInputChange"
					/>
				</button>
			</ButtonStyled>
		</div>

		<Checkbox v-model="ctx.hasLicensePermission.value">
			<span class="max-w-[90%] text-left text-primary">
				Do you have the appropriate licenses to redistribute all content in this Modpack?
				<NuxtLink
					to="https://support.modrinth.com/en/articles/8797527-obtaining-modpack-permissions"
					external
					target="_blank"
					class="font-medium text-blue underline"
				>
					Learn more
				</NuxtLink>
			</span>
		</Checkbox>
	</div>
</template>

<script setup lang="ts">
import { ArrowLeftRightIcon, FileIcon } from '@modrinth/assets'
import { ButtonStyled, Checkbox, DropzoneFileInput } from '@modrinth/ui'

import { injectServerCompatibilityContext } from '../manage-server-compatibility-modal'

const ctx = injectServerCompatibilityContext()

const fileInput = useTemplateRef<HTMLInputElement>('fileInput')

function handleFileUpload(files: File[]) {
	if (files.length > 0) {
		ctx.customModpackFile.value = files[0]
	}
}

function handleFileInputChange(e: Event) {
	const target = e.target as HTMLInputElement
	const file = target.files?.[0]
	if (file) {
		ctx.customModpackFile.value = file
	}
	target.value = ''
}
</script>
