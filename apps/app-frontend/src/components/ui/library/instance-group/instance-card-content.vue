<script setup lang="ts">
import { PageRoundIcon } from '@modrinth/assets'
import { Avatar } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed } from 'vue'

import InstanceFileIcon from '@/assets/icons/instance-file.svg'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instance: GameInstance
}>()

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
	<Avatar
		v-if="instance.icon_path"
		class="pointer-events-none !border-none ![background-color:color-mix(in_srgb,var(--color-contrast)_5%,transparent)] !rounded-[22px] !p-0 !rounded-br-[38px] !absolute -top-[26px] right-[20px] opacity-50 [mask-image:linear-gradient(135deg,transparent_16%,black_100%)]"
		size="84px"
		:src="convertFileSrc(instance.icon_path)"
		:tint-by="instance.id"
		alt=""
		no-shadow
	/>
	<PageRoundIcon
		aria-hidden="true"
		class="pointer-events-none absolute -top-[52px] right-[0px] size-[124px] [mask-image:linear-gradient(135deg,transparent_16%,black_100%)]"
		:class="{ 'opacity-[0.03]': instance.icon_path, 'opacity-5': !instance.icon_path }"
	/>
	<div class="relative z-[1] flex min-w-0 flex-1 items-center gap-2 pr-20">
		<slot name="leading" :instance-type="instanceType">
			<div
				class="flex size-10 shrink-0 flex-col items-center gap-px overflow-clip rounded-[14px] px-[3px] py-0.5 text-primary"
			>
				<InstanceFileIcon class="h-[21px] w-[31px] shrink-0 text-primary [&_path]:fill-current" />
				<span class="h-3.5 text-sm font-extrabold leading-[13px]">{{ instanceType }}</span>
			</div>
		</slot>
		<div class="flex min-w-0 flex-1 flex-col justify-center gap-1">
			<p class="m-0 truncate text-base font-semibold leading-5 text-contrast">
				{{ instance.name }}
			</p>
			<p class="m-0 truncate text-sm font-medium capitalize leading-[18px] text-primary">
				{{ instance.loader }} {{ instance.game_version }}
			</p>
		</div>
	</div>
</template>
