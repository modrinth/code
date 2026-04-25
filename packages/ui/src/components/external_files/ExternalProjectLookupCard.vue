<script setup lang="ts">
import {
	ClipboardCopyIcon,
	CurseForgeIcon,
	FileIcon,
	LinkIcon,
	UnknownIcon,
} from '@modrinth/assets'
import { Menu } from 'floating-vue'
import { computed } from 'vue'

import { ButtonStyled, CopyCode } from '#ui/components'

import ExternalProjectLicenseStateTag from './ExternalProjectLicenseStateTag.vue'
import type { ExternalLicenseStatus } from './types.ts'

const props = defineProps<{
	title: string | null
	link: string | null
	state: ExternalLicenseStatus
	proof: string | null
	notes: string | null
	last_updated?: string
	last_updated_by?: string
	cf_id?: number | null
	files: {
		sha1: string
		name: string | null
	}[]
}>()

const lastEditedText = computed(() =>
	props.last_updated
		? `Last edited on ${props.last_updated} by ${props.last_updated_by ?? 'unknown'}`
		: '',
)

async function copyProjectLink() {
	if (!props.link) return
	await navigator.clipboard.writeText(props.link)
}
</script>

<template>
	<div class="bg-surface-3 p-4 rounded-2xl flex flex-col gap-3">
		<div class="flex gap-4 justify-between">
			<div class="flex flex-col gap-2">
				<span class="text-contrast font-semibold">{{ title }}</span>
				<div v-if="!!link" class="flex gap-2 items-center">
					<a
						class="flex items-center gap-2 font-medium hover:underline focus:underline w-fit text-blue"
						:href="link"
						target="_blank"
					>
						<template v-if="link?.includes('curseforge.com')">
							<CurseForgeIcon class="size-5 shrink-0" />
							CurseForge project
						</template>
						<template v-else> <LinkIcon class="size-5 shrink-0" /> Project link </template>
					</a>
					<ButtonStyled circular type="transparent" size="small">
						<button v-tooltip="'Copy link'" @click="copyProjectLink">
							<ClipboardCopyIcon />
						</button>
					</ButtonStyled>
				</div>
			</div>
			<slot name="actions" />
		</div>

		<div class="grid grid-cols-[auto_1fr] items-center gap-x-4 gap-y-2">
			<div class="font-medium">Allowed:</div>
			<div>
				<ExternalProjectLicenseStateTag :state="state" />
			</div>
			<div class="font-medium">Proof:</div>
			<div>{{ proof ?? 'N/A' }}</div>
			<template v-if="cf_id">
				<div class="font-medium">CurseForge ID:</div>
				<CopyCode :text="`${cf_id}`" />
			</template>
			<div class="font-medium">Notes:</div>
			<div>{{ notes ?? 'N/A' }}</div>
		</div>
		<div class="bg-surface-2 p-4 rounded-2xl flex flex-col gap-3">
			<span class="text-contrast font-semibold">Files</span>
			<span v-if="!(files?.length > 0)" class="text-secondary">
				No files available for external project.
			</span>
			<div v-else class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-2">
				<Menu v-for="file in files" :key="file.sha1" :delay="{ hide: 50, show: 0 }">
					<div
						class="line-clamp-1 truncate px-2 py-1 flex gap-2 rounded-xl items-center border-solid border-2 border-surface-5 text-sm font-medium text-secondary"
					>
						<FileIcon class="shrink-0 size-4" /> {{ file.name ?? file.sha1 }}
					</div>
					<template #popper>
						<div class="text-primary p-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
							<div>First identified name:</div>
							<div v-if="file.name" class="text-sm">
								{{ file.name }}
							</div>
							<div v-else class="text-secondary flex items-center gap-2 text-sm">
								<UnknownIcon /> Unknown
							</div>
							<div>SHA-1:</div>
							<div class="text-sm"><CopyCode :text="file.sha1" /></div>
						</div>
					</template>
				</Menu>
			</div>
		</div>
		<div v-if="last_updated" class="pt-4 border-t-[1px] border-solid border-surface-5">
			{{ lastEditedText }}
		</div>
	</div>
</template>
