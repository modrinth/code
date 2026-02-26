<template>
	<div class="flex gap-1.5 items-center justify-between px-3 py-1.5 rounded-2xl bg-surface-2">
		<div class="grid grid-cols-[auto_1fr] gap-1.5 items-center">
			<Avatar :src="icon" size="34px" class="!rounded-xl" />
			<div class="flex flex-col items-start overflow-hidden">
				<div
					v-tooltip="name"
					class="truncate font-semibold text-sm"
					:class="onclickName ? 'hover:underline cursor-pointer' : ''"
					@click="onclickName"
				>
					{{ name }}
				</div>
				<div
					v-tooltip="versionNumber"
					class="truncate font-medium text-sm"
					:class="onclickVersion ? 'hover:underline cursor-pointer' : ''"
					@click="onclickVersion"
				>
					{{ versionNumber }}
				</div>
			</div>
		</div>
		<ButtonStyled v-if="onclickDownload" circular>
			<button @click="onclickDownload" v-tooltip="'Download modpack'">
				<DownloadIcon />
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets/generated-icons'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'

defineProps<{
	name: string
	versionNumber: string
	icon?: string
	onclickName?: () => void
	onclickVersion?: () => void
	onclickDownload?: () => void
}>()
</script>
