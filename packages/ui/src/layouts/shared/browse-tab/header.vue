<script setup lang="ts">
import { LeftArrowIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { useServerImage } from '#ui/composables/use-server-image'
import { formatLoaderLabel } from '#ui/utils/loaders'

import SelectedProjectsLeaveModal from './components/SelectedProjectsLeaveModal.vue'
import { injectBrowseManager } from './providers/browse-manager'
import type { BrowseInstallContext } from './types'

const MEDAL_ICON_URL = 'https://cdn-raw.modrinth.com/medal_icon.webp'

const router = useRouter()
const props = defineProps<{
	installContext?: BrowseInstallContext | null
}>()
type SelectedProjectsLeaveResult = 'cancel' | 'discard' | 'install'

const ctx = injectBrowseManager(null)
const installContext = computed(() => props.installContext ?? ctx?.installContext?.value ?? null)
const selectedProjectsLeaveModal = ref<InstanceType<typeof SelectedProjectsLeaveModal>>()

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

const selectedCount = computed(() => installContext.value?.selectedProjects?.length ?? 0)
const isInstallingSelected = computed(() => installContext.value?.isInstallingSelected ?? false)

async function handleBack() {
	const context = installContext.value
	if (!context) return

	if (selectedCount.value > 0 && !isInstallingSelected.value) {
		const result = await selectedProjectsLeaveModal.value?.prompt()
		await handleSelectedProjectsLeaveResult(result ?? 'cancel', context)
		return
	}

	const shouldNavigate = await context.onBack?.()
	if (shouldNavigate === false) return

	await router.push(context.backUrl)
}

async function handleSelectedProjectsLeaveResult(
	result: SelectedProjectsLeaveResult,
	context: BrowseInstallContext,
) {
	if (result === 'cancel') return
	if (result === 'install') {
		const shouldNavigate = await context.installSelected?.()
		if (shouldNavigate === false) return
		return
	}

	if (context.discardSelectedAndBack) {
		await context.discardSelectedAndBack()
		return
	}

	await (context.clearSelected ?? context.clearQueued)?.()
	await router.push(context.backUrl)
}
</script>

<template>
	<template v-if="installContext">
		<SelectedProjectsLeaveModal
			ref="selectedProjectsLeaveModal"
			:count="selectedCount"
			:installing="isInstallingSelected"
		/>
		<div class="flex flex-col gap-2">
			<div class="flex flex-wrap items-center justify-between gap-4">
				<div class="flex min-w-0 items-center gap-4">
					<ButtonStyled circular size="large">
						<button :aria-label="installContext.backLabel" @click="handleBack">
							<LeftArrowIcon />
						</button>
					</ButtonStyled>

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
			</div>
		</div>
		<Admonition v-if="installContext.warning" type="warning" class="mb-1">
			{{ installContext.warning }}
		</Admonition>
	</template>
</template>
