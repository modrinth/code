<script setup lang="ts">
import { DownloadIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, inject, onMounted, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import LibrarySection from '@/components/ui/library/index.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { instance_listener } from '@/helpers/events'
import { list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'
import { useBreadcrumbs } from '@/store/breadcrumbs'

import appIcon from '../../../app/icons/128x128.png'

const { handleError } = injectNotificationManager()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const showCreationModal = inject<() => void>('showCreationModal')
const showImportModal = inject<() => void>('showImportModal')
const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const instances = ref<GameInstance[]>([])

const recentInstances = computed(() =>
	instances.value
		.filter((x) => x.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

const offline = ref<boolean>(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

async function fetchInstances() {
	try {
		instances.value = await list()
	} catch (error: unknown) {
		handleError(error)
	}
}

if (hasCreatedInstance.value) {
	await fetchInstances()
}

const unlistenInstance = await instance_listener(fetchInstances)

function handleQuickCreate(event: KeyboardEvent) {
	const target = event.target as HTMLElement | null
	if (
		event.key.toLowerCase() !== 'n' ||
		event.repeat ||
		event.metaKey ||
		event.ctrlKey ||
		event.altKey ||
		target?.isContentEditable ||
		['INPUT', 'TEXTAREA', 'SELECT'].includes(target?.tagName ?? '')
	) {
		return
	}

	if (isReady.value && !hasCreatedInstance.value && !offline.value) {
		event.preventDefault()
		showCreationModal?.()
	}
}

onMounted(() => window.addEventListener('keydown', handleQuickCreate))

onUnmounted(() => {
	unlistenInstance()
	window.removeEventListener('keydown', handleQuickCreate)
})
</script>

<template>
	<div
		v-if="isReady && !hasCreatedInstance"
		class="grid min-h-full grid-rows-[minmax(30.8125rem,1fr)_auto] px-6 pb-6 pt-16"
	>
		<div class="relative flex justify-center items-center">
			<div
				class="pointer-events-none absolute left-1/2 top-0 h-[29.875rem] w-[min(25.9375rem,80vw)] -translate-x-1/2 rounded-2xl bg-[radial-gradient(circle,var(--color-surface-5)_0.5px,transparent_0.75px)] opacity-60 [background-size:0.5625rem_0.5625rem] [mask-image:radial-gradient(ellipse_at_center,black_10%,transparent_68%)] [@media(max-height:700px)]:h-[23rem]"
				aria-hidden="true"
			/>
			<div class="relative flex flex-col items-center gap-6">
				<div
					class="flex h-[6.25rem] w-[6.25rem] items-center justify-center overflow-hidden rounded-3xl bg-[linear-gradient(180deg,rgb(37_114_79_/_50%)_0%,rgb(15_64_36_/_50%)_100%),linear-gradient(-14deg,rgb(0_0_0_/_37%)_8%,transparent_86%),black] shadow-[0_0_0_1.5px_#07180d,0_1.5rem_3rem_rgb(0_0_0_/_3%),0_0.625rem_1.125rem_rgb(0_0_0_/_3%),0_0.3125rem_0.5rem_rgb(0_0_0_/_4%),0_0.125rem_0.25rem_rgb(0_0_0_/_4%),0_0.625rem_13.75rem_rgb(27_217_106_/_25%)]"
				>
					<img :src="appIcon" alt="" class="h-[6rem] w-[6rem]" />
				</div>
				<div class="flex flex-col items-center gap-2">
					<h1 class="m-0 flex items-center gap-2 text-2xl font-semibold leading-8 text-contrast">
						Welcome to Modrinth
					</h1>
					<p class="m-0 text-center text-base leading-6 text-primary">
						Minecraft was never meant to fit in one jar.
					</p>
				</div>
				<div class="flex w-72 flex-col items-center gap-4">
					<ButtonStyled color="brand" size="large">
						<button class="!shadow-none" :disabled="offline" @click="showCreationModal?.()">
							<PlusIcon />
							Create an instance
						</button>
					</ButtonStyled>
					<span class="flex items-center gap-1 text-sm leading-5 text-secondary">
						Press
						<kbd
							class="inline-flex h-5 min-w-5 items-center justify-center rounded-md border border-solid border-surface-5 bg-button-bg px-1 text-xs font-normal leading-4 text-primary"
							>N</kbd
						>
						to quick create an instance
					</span>
				</div>
			</div>
		</div>
		<div class="flex flex-col items-center justify-center gap-4 text-sm leading-5 text-secondary">
			<span class="whitespace-nowrap">Escaping another launcher?</span>
			<ButtonStyled>
				<button class="!shadow-none" :disabled="offline" @click="showImportModal?.()">
					<DownloadIcon />
					Import from launcher
				</button>
			</ButtonStyled>
		</div>
	</div>
	<div v-else-if="isReady" class="flex flex-col gap-6 p-6">
		<h1 v-if="recentInstances?.length > 0" class="m-0 text-2xl font-extrabold">Welcome back!</h1>
		<h1 v-else class="m-0 text-2xl font-extrabold">Welcome to Modrinth App!</h1>
		<RecentWorldsList :recent-instances="recentInstances" />
		<LibrarySection :instances="instances" />
	</div>
</template>
