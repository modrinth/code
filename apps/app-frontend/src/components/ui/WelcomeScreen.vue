<script setup lang="ts">
import { DownloadIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { inject, onMounted, onUnmounted, ref } from 'vue'

import appIcon from '../../../../app/icons/128x128.png'

const showCreationModal = inject<() => void>('showCreationModal')
const showImportModal = inject<() => void>('showImportModal')

const offline = ref(!navigator.onLine)

function handleOffline() {
	offline.value = true
}

function handleOnline() {
	offline.value = false
}

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

	if (!offline.value) {
		event.preventDefault()
		showCreationModal?.()
	}
}

onMounted(() => {
	window.addEventListener('offline', handleOffline)
	window.addEventListener('online', handleOnline)
	window.addEventListener('keydown', handleQuickCreate)
})

onUnmounted(() => {
	window.removeEventListener('offline', handleOffline)
	window.removeEventListener('online', handleOnline)
	window.removeEventListener('keydown', handleQuickCreate)
})
</script>

<template>
	<div class="grid min-h-full grid-rows-[minmax(30.8125rem,1fr)_auto] px-6 pb-6 pt-16">
		<div class="relative flex items-center justify-center">
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
</template>
