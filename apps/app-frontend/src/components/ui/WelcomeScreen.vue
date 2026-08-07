<script setup lang="ts">
import { ImportIcon, PlusIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import frog from '@modrinth/ui/src/assets/welcome/frog.png'
import { inject, onMounted, onUnmounted, ref } from 'vue'

import modrinthSocialIcon from '../../assets/welcome/modrinth-social-icon.png'

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
	<div class="flex flex-col min-h-full px-6 pb-6 pt-16">
		<div class="relative flex grow items-center justify-center">
			<div class="relative flex flex-col items-center gap-6">
				<div
					class="dot-pattern pointer-events-none absolute left-1/2 -top-52 h-[29.875rem] w-[min(25.9375rem,80vw)] -translate-x-1/2 rounded-2xl [@media(max-height:700px)]:h-[23rem]"
					aria-hidden="true"
				/>
				<div class="relative h-[6.25rem] w-[6.25rem]">
					<img
						:src="modrinthSocialIcon"
						alt=""
						class="welcome-artwork pointer-events-none absolute left-1/2 top-1/2 h-[33.75rem] w-[33.75rem] max-w-none -translate-x-1/2 -translate-y-1/2"
					/>
				</div>
				<div class="flex flex-col items-center gap-2">
					<h1 class="m-0 flex items-center gap-2 text-2xl font-semibold leading-8 text-contrast">
						Welcome to Modrinth
						<img :src="frog" alt="" class="h-8 w-8 [image-rendering:pixelated]" />
					</h1>
					<p class="m-0 text-center text-base leading-6 text-primary">Ready to start playing?</p>
				</div>
				<div class="flex w-72 flex-col items-center gap-4">
					<Button
						type="colored"
						color="brand"
						size="lg"
						class="!shadow-none"
						:disabled="offline"
						@click="showCreationModal?.()"
					>
						<PlusIcon />
						Create an instance
					</Button>
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
		<div
			class="flex flex-col h-max items-center justify-end gap-4 text-sm leading-5 text-secondary"
		>
			<span class="whitespace-nowrap">Escaping another launcher?</span>
			<Button size="lg" class="!font-medium" :disabled="offline" @click="showImportModal?.()">
				<ImportIcon />
				Import from launcher
			</Button>
		</div>
	</div>
</template>

<style scoped>
.welcome-artwork {
	-webkit-mask-image: radial-gradient(circle, black 45%, transparent 72%);
	mask-image: radial-gradient(circle, black 45%, transparent 72%);
}

.dot-pattern {
	background-image: radial-gradient(
		circle,
		color-mix(in srgb, var(--color-text-primary) 25%, transparent) 0.5px,
		transparent 0.75px
	);
	background-size: 0.5625rem 0.5625rem;
	opacity: 0.8;
	-webkit-mask-image: radial-gradient(ellipse at center, black 10%, transparent 68%);
	mask-image: radial-gradient(ellipse at center, black 10%, transparent 68%);
	-webkit-mask-repeat: no-repeat;
	mask-repeat: no-repeat;
}
</style>
