<script setup lang="ts">
import { Avatar, truncatedTooltip } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { getInstanceIconUrl } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		selected?: boolean
	}>(),
	{
		selected: false,
	},
)

const iconSrc = computed(() => getInstanceIconUrl(props.instance.icon_path))
const appSettings = useAppSettings()
const compactMode = computed(() => appSettings.getFeatureFlag('compact_instance_cards'))

const nameRef = ref<HTMLElement | null>(null)
const versionRef = ref<HTMLElement | null>(null)
</script>

<template>
	<div
		class="relative flex w-full min-w-0 select-none overflow-clip border border-solid bg-surface-3 text-left transition-all"
		:class="{
			'flex-row items-center justify-start gap-2.5 rounded-xl p-2.5': compactMode,
			'flex-col items-start justify-end gap-3 rounded-[20px] p-3': !compactMode,
			'[border-color:color-mix(in_srgb,var(--color-text-primary)_40%,transparent)] brightness-110':
				selected,
			'border-surface-4': !selected,
		}"
	>
		<div
			class="relative flex shrink-0 items-center overflow-clip"
			:class="compactMode ? 'size-10 rounded-lg' : 'aspect-square min-w-full rounded-2xl'"
		>
			<Avatar
				class="pointer-events-none outline-none"
				:class="compactMode ? '!rounded-lg' : '!rounded-2xl'"
				size="100%"
				:src="iconSrc"
				:tint-by="instance.id"
				alt=""
				no-shadow
			/>
			<slot name="loading" :compact="compactMode" />
			<div
				class="absolute z-[1] flex items-center justify-center"
				:class="compactMode ? 'inset-0' : 'bottom-1.5 right-1.5 size-12'"
			>
				<slot name="leading" :compact="compactMode" />
			</div>
		</div>
		<div
			class="flex min-w-0 w-full flex-col items-start justify-center gap-1 px-0.5"
			:class="{ 'pr-10': compactMode }"
		>
			<p
				ref="nameRef"
				v-tooltip="truncatedTooltip(nameRef, instance.name)"
				class="m-0 w-full truncate text-base font-semibold leading-5 text-contrast"
			>
				{{ instance.name }}
			</p>
			<p
				ref="versionRef"
				v-tooltip="truncatedTooltip(versionRef, `${instance.loader} ${instance.game_version}`)"
				class="m-0 w-full truncate text-sm font-medium capitalize leading-[18px] text-primary"
			>
				{{ instance.loader }} {{ instance.game_version }}
			</p>
		</div>
		<slot name="overlay" :compact="compactMode" />
	</div>
</template>
