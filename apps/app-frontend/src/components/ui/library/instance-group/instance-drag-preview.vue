<script setup lang="ts">
import { Avatar, TagItem } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed } from 'vue'

import InstanceFileIcon from '@/assets/icons/instance-file.svg'
import type { GameInstance } from '@/helpers/types'

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		count?: number
	}>(),
	{
		count: 1,
	},
)

const instanceType = computed(() => {
	if (
		props.instance.link?.type === 'server_project' ||
		props.instance.link?.type === 'server_project_modpack'
	) {
		return 'SRV'
	}

	return props.instance.link?.type === 'modrinth_modpack' ? 'MPK' : 'CST'
})
</script>

<template>
	<div aria-hidden="true" class="relative w-full select-none">
		<div
			v-if="count > 1"
			class="absolute inset-x-3 -bottom-2 top-2 rounded-[20px] border border-solid border-surface-5 bg-surface-2 opacity-60 shadow-md"
		/>
		<div
			v-if="count > 1"
			class="absolute inset-x-1.5 -bottom-1 top-1 rounded-[20px] border border-solid border-surface-4 bg-surface-3 opacity-80 shadow-md"
		/>
		<div
			class="relative flex min-h-[76px] w-full items-center justify-center gap-2 overflow-clip rounded-[20px] border border-solid border-surface-4 bg-surface-3 p-4 text-left opacity-90 shadow-lg"
		>
			<Avatar
				class="pointer-events-none !border-none !bg-transparent !rounded-[26px] !rounded-br-[42px] !absolute -top-[40px] right-[18px] opacity-50 [mask-image:linear-gradient(135deg,transparent_16%,black_100%)]"
				size="100px"
				:src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
				:tint-by="instance.id"
				alt=""
				no-shadow
			/>
			<TagItem
				v-if="count > 1"
				class="!absolute right-3 top-3 z-[2] border-surface-5 bg-surface-2 font-semibold tabular-nums text-contrast"
			>
				{{ count }}
			</TagItem>
			<div class="relative z-[1] flex min-w-0 flex-1 items-center gap-2 pr-20">
				<div
					class="flex size-10 shrink-0 flex-col items-center gap-px overflow-clip rounded-[14px] px-[3px] py-0.5 text-primary"
				>
					<InstanceFileIcon class="h-[21px] w-[31px] shrink-0 text-primary [&_path]:fill-current" />
					<span class="h-3.5 text-sm font-extrabold leading-[13px]">{{ instanceType }}</span>
				</div>
				<div class="flex min-w-0 flex-1 flex-col justify-center gap-1">
					<p class="m-0 truncate text-base font-semibold leading-5 text-contrast">
						{{ instance.name }}
					</p>
					<p class="m-0 truncate text-sm font-medium capitalize leading-[18px] text-primary">
						{{ instance.loader }} {{ instance.game_version }}
					</p>
				</div>
			</div>
		</div>
	</div>
</template>
