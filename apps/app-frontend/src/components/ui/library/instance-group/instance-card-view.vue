<script setup lang="ts">
import { PageRoundIcon } from '@modrinth/assets'
import { Avatar } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, type CSSProperties, ref } from 'vue'

import InstanceFileIcon from '@/assets/icons/instance-file.svg'
import type { GameInstance } from '@/helpers/types'

const avatarSize = '84px'
const avatarStyles: CSSProperties = {
	top: '-26px',
	right: '20px',
	width: avatarSize,
	height: avatarSize,
	borderRadius: '22px 22px 38px 22px',
}
const avatar = ref<InstanceType<typeof Avatar> | null>(null)

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		selected?: boolean
	}>(),
	{
		selected: false,
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

const iconSrc = computed(() =>
	props.instance.icon_path ? convertFileSrc(props.instance.icon_path) : undefined,
)
const avatarFailed = computed(() => avatar.value?.failed ?? false)
const hasVisibleIcon = computed(() => Boolean(iconSrc.value) && !avatarFailed.value)
</script>

<template>
	<div
		class="relative flex min-h-[76px] w-full select-none items-center justify-center gap-2 overflow-clip rounded-[20px] border border-solid bg-surface-3 p-4 text-left transition-all"
		:class="{
			'[border-color:color-mix(in_srgb,var(--color-text-primary)_20%,transparent)] brightness-110':
				selected,
			'border-surface-4': !selected,
		}"
	>
		<PageRoundIcon
			aria-hidden="true"
			class="pointer-events-none absolute -top-[52px] right-[0px] size-[124px] [mask-image:linear-gradient(135deg,transparent_16%,black_100%)]"
			:class="{ 'opacity-[0.03]': hasVisibleIcon, 'opacity-5': !hasVisibleIcon }"
		/>
		<div
			v-if="hasVisibleIcon"
			class="pointer-events-none absolute bg-surface-3"
			:style="avatarStyles"
		/>
		<Avatar
			v-if="iconSrc"
			ref="avatar"
			class="pointer-events-none !border-none ![background-color:color-mix(in_srgb,var(--color-contrast)_5%,transparent)] !p-0 !absolute opacity-50 [mask-image:linear-gradient(135deg,transparent_16%,black_100%)]"
			:class="{ hidden: avatarFailed }"
			:style="avatarStyles"
			:size="avatarSize"
			:src="iconSrc"
			:tint-by="instance.id"
			alt=""
			no-shadow
		/>
		<div class="relative z-[1] flex min-w-0 flex-1 items-center gap-2 pr-20">
			<slot name="leading" :instance-type="instanceType">
				<div
					class="flex size-10 shrink-0 flex-col items-center gap-px overflow-clip rounded-[14px] px-[3px] py-0.5 text-primary"
				>
					<InstanceFileIcon class="h-[21px] w-[31px] shrink-0 text-primary [&_path]:fill-current" />
					<span class="h-3.5 text-sm font-extrabold leading-[13px]">
						{{ instanceType }}
					</span>
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
		<slot name="overlay" />
	</div>
</template>
