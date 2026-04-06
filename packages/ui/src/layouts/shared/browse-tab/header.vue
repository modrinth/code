<script setup lang="ts">
import { GameIcon, LeftArrowIcon, MinecraftServerIcon } from '@modrinth/assets'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { useServerImage } from '#ui/composables/use-server-image'

import { injectBrowseManager } from './providers/browse-manager'

const MEDAL_ICON_URL = 'https://cdn-raw.modrinth.com/medal_icon.webp'

const ctx = injectBrowseManager()
const router = useRouter()
const installContext = computed(() => ctx.installContext?.value ?? null)

const serverId = computed(() => installContext.value?.serverId ?? '')
const upstream = computed(() => installContext.value?.upstream ?? null)

const { image: fetchedIcon } = useServerImage(serverId, upstream, {
	enabled: computed(() => !!installContext.value?.serverId),
})

const iconSrc = computed(() => {
	if (installContext.value?.isMedal) return MEDAL_ICON_URL
	return fetchedIcon.value ?? installContext.value?.iconSrc ?? MinecraftServerIcon
})
</script>

<template>
	<template v-if="installContext">
		<div
			class="flex flex-wrap items-center justify-between gap-3 mb-2 border-0 border-b border-solid border-divider pb-4"
		>
			<div class="flex items-center gap-2">
				<Avatar :src="iconSrc" size="48px" />
				<div class="flex flex-col gap-1">
					<span
						class="font-extrabold text-contrast"
						:class="ctx.variant === 'web' ? 'text-base' : 'text-lg'"
					>
						{{ installContext.name }}
					</span>
					<span class="flex items-center gap-2 text-sm font-semibold text-secondary">
						<GameIcon class="h-5 w-5 text-secondary" />
						{{ installContext.loader }} {{ installContext.gameVersion }}
					</span>
				</div>
			</div>
			<ButtonStyled>
				<button @click="router.push(installContext.backUrl)">
					<LeftArrowIcon />
					{{ installContext.backLabel }}
				</button>
			</ButtonStyled>
		</div>
		<h1 class="m-0 mb-1 text-xl font-extrabold">{{ installContext.heading }}</h1>
		<Admonition v-if="installContext.warning" type="warning" class="mb-1">
			{{ installContext.warning }}
		</Admonition>
	</template>
</template>
