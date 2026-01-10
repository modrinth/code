<template>
	<div class="flex w-full flex-col items-center justify-center gap-6 py-8">
		<div class="text-center">
			<h3 class="text-lg font-semibold text-contrast">Uploading version</h3>
			<p class="mt-1 text-secondary">Please wait while your files are being uploaded...</p>
		</div>

		<div class="w-full max-w-md">
			<div class="mb-2 flex justify-between text-sm">
				<span class="font-semibold">{{ Math.round(uploadProgress.progress * 100) }}%</span>
				<span class="text-secondary">
					{{ formatBytes(uploadProgress.loaded) }} / {{ formatBytes(uploadProgress.total) }}
				</span>
			</div>
			<div class="h-2 w-full rounded-full bg-button-bg">
				<div
					class="h-2 rounded-full bg-brand transition-all duration-300 ease-out"
					:style="{ width: `${uploadProgress.progress * 100}%` }"
				></div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { formatBytes } from '@modrinth/utils'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { uploadProgress } = injectManageVersionContext()
</script>
