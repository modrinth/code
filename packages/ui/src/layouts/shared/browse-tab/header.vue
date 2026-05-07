<script setup lang="ts">
import { BoxIcon, LeftArrowIcon, XIcon } from '@modrinth/assets'
import { Tooltip } from 'floating-vue'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { useServerImage } from '#ui/composables/use-server-image'
import { formatLoaderLabel } from '#ui/utils/loaders'

import { injectBrowseManager } from './providers/browse-manager'
import type { BrowseInstallContext } from './types'

const MEDAL_ICON_URL = 'https://cdn-raw.modrinth.com/medal_icon.webp'

const router = useRouter()
const props = defineProps<{
	installContext?: BrowseInstallContext | null
}>()
const ctx = injectBrowseManager(null)
const installContext = computed(() => props.installContext ?? ctx?.installContext?.value ?? null)

const serverId = computed(() => installContext.value?.serverId ?? '')
const upstream = computed(() => installContext.value?.upstream ?? null)

const { image: fetchedIcon } = useServerImage(serverId, upstream, {
	enabled: computed(() => !!installContext.value?.serverId),
})

const iconSrc = computed(() => {
	if (installContext.value?.isMedal) return MEDAL_ICON_URL
	return fetchedIcon.value ?? installContext.value?.iconSrc ?? null
})

const metadataItems = computed(() => {
	const context = installContext.value
	if (!context) return []
	return [
		context.heading,
		context.gameVersion ? `MC ${context.gameVersion}` : '',
		context.loader ? formatLoaderLabel(context.loader) : '',
	].filter(Boolean)
})

const queuedCount = computed(() => installContext.value?.queuedCount ?? 0)
const returnToInstallTooltipDismissed = ref(false)
const showReturnToInstallTooltip = computed(
	() =>
		queuedCount.value > 0 &&
		!!installContext.value?.clearQueued &&
		!returnToInstallTooltipDismissed.value,
)
const queuedLabel = computed(() => {
	if (installContext.value?.queuedLabel) return installContext.value.queuedLabel
	return `${queuedCount.value} ${queuedCount.value === 1 ? 'Mod' : 'Mods'}`
})

async function handleBack() {
	const context = installContext.value
	if (!context) return

	const shouldNavigate = await context.onBack?.()
	if (shouldNavigate === false) return

	await router.push(context.backUrl)
}

async function clearQueued() {
	await installContext.value?.clearQueued?.()
}
</script>

<template>
	<template v-if="installContext">
		<div class="flex flex-col gap-2">
			<div class="flex flex-wrap items-center justify-between gap-4">
				<div class="flex min-w-0 items-center gap-4">
					<Tooltip
						theme="dismissable-prompt"
						:triggers="[]"
						:shown="showReturnToInstallTooltip"
						:auto-hide="false"
						placement="bottom-start"
						popper-class="!z-[60]"
					>
						<ButtonStyled circular size="large">
							<button :aria-label="installContext.backLabel" @click="handleBack">
								<LeftArrowIcon />
							</button>
						</ButtonStyled>
						<template #popper>
							<div class="grid max-w-64 gap-1">
								<div class="flex items-center justify-between gap-4">
									<h3 class="m-0 text-base font-bold text-contrast">Return to install content</h3>
									<ButtonStyled size="small" circular>
										<button v-tooltip="'Dismiss'" @click="returnToInstallTooltipDismissed = true">
											<XIcon aria-hidden="true" />
										</button>
									</ButtonStyled>
								</div>
								<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
									The content you have queued will be installed when you return to your server
									panel.
								</p>
							</div>
						</template>
					</Tooltip>

					<Avatar v-if="iconSrc" :src="iconSrc" size="48px" class="shrink-0" />

					<div class="flex min-w-0 flex-col justify-center gap-1">
						<h1 class="m-0 truncate text-2xl font-semibold leading-8 text-contrast">
							{{ installContext.name }}
						</h1>
						<div
							v-if="metadataItems.length"
							class="flex flex-wrap items-center gap-2 text-base font-medium leading-6 text-primary"
						>
							<template v-for="(item, index) in metadataItems" :key="item">
								<span
									v-if="index > 0"
									class="h-1.5 w-1.5 shrink-0 rounded-full bg-current opacity-60"
								/>
								<span>{{ item }}</span>
							</template>
						</div>
					</div>
				</div>

				<div
					v-if="queuedCount > 0 && installContext.clearQueued"
					class="flex shrink-0 items-center gap-4 text-base font-medium leading-6 text-primary"
				>
					<div class="flex items-center gap-1.5">
						<span>Queued</span>
						<BoxIcon class="h-5 w-5" />
						<span>{{ queuedLabel }}</span>
					</div>
					<div class="h-[18px] w-px bg-surface-5" />
					<ButtonStyled type="transparent">
						<button type="button" @click="clearQueued">Clear</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<Admonition v-if="installContext.warning" type="warning" class="mb-1">
			{{ installContext.warning }}
		</Admonition>
	</template>
</template>
