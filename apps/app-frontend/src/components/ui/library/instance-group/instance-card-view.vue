<script setup lang="ts">
import { Avatar, truncatedTooltip } from '@modrinth/ui'
import { computed, ref } from 'vue'

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

const nameRef = ref<HTMLElement | null>(null)
const versionRef = ref<HTMLElement | null>(null)
</script>

<template>
	<div
		class="relative flex w-full min-w-0 select-none flex-col items-start justify-end gap-3 overflow-clip rounded-[20px] border border-solid bg-surface-3 p-3 text-left transition-all"
		:class="{
			'[border-color:color-mix(in_srgb,var(--color-text-primary)_40%,transparent)] brightness-110':
				selected,
			'border-surface-4': !selected,
		}"
	>
		<div
			class="relative flex aspect-square min-w-full shrink-0 items-center overflow-clip rounded-2xl"
		>
			<Avatar
				class="pointer-events-none !rounded-2xl outline-none"
				size="100%"
				:src="iconSrc"
				:tint-by="instance.id"
				alt=""
				no-shadow
			/>
			<slot name="loading" />
			<div class="absolute bottom-1.5 right-1.5 z-[1] flex size-12 items-center justify-center">
				<slot name="leading" />
			</div>
		</div>
		<div class="flex min-w-0 w-full flex-col items-start justify-center gap-1 px-0.5">
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
		<slot name="overlay" />
	</div>
</template>
