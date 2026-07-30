<script setup lang="ts">
import { ImportIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { inject, onMounted, onUnmounted, ref } from 'vue'

import frog from '../../assets/welcome/frog.png'
import iconLights from '../../assets/welcome/icon-lights.svg?url'
import iconLogo from '../../assets/welcome/icon-logo.svg?url'
import iconTexture from '../../assets/welcome/icon-texture.png'

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
	<div class="grid min-h-full grid-rows-[30.8125rem_1fr_auto] px-6 pb-6 pt-16">
		<div class="relative flex items-end justify-center">
			<div
				class="dot-pattern pointer-events-none absolute left-1/2 top-0 h-[29.875rem] w-[min(25.9375rem,80vw)] -translate-x-1/2 rounded-2xl [@media(max-height:700px)]:h-[23rem]"
				aria-hidden="true"
			/>
			<div class="relative flex flex-col items-center gap-6">
				<div
					class="welcome-icon relative h-[6.25rem] w-[6.25rem] overflow-hidden rounded-3xl border-[1.5px] border-solid bg-surface-1"
					style="border-color: color-mix(in srgb, var(--color-text-primary) 15%, transparent)"
				>
					<div
						class="absolute inset-0 bg-[linear-gradient(180deg,rgb(37_114_79)_0%,rgb(15_64_36)_100%)] opacity-50"
					/>
					<div
						class="absolute inset-0 bg-[linear-gradient(-14deg,rgb(0_0_0_/_37%)_8%,transparent_86%)]"
					/>
					<img
						:src="iconTexture"
						alt=""
						class="absolute left-1/2 top-1/2 h-[12.70625rem] w-[19.96875rem] max-w-none -translate-x-1/2 -translate-y-1/2 object-cover opacity-40 mix-blend-luminosity"
					/>
					<img :src="iconLogo" alt="" class="absolute inset-[13%] h-[74%] w-[74%]" />
					<img
						:src="iconLights"
						alt=""
						class="absolute left-[0.4775rem] top-[0.4775rem] h-[0.4804rem] w-[1.2714rem]"
					/>
				</div>
				<div class="flex flex-col items-center gap-2">
					<h1 class="m-0 flex items-center gap-2 text-2xl font-semibold leading-8 text-contrast">
						Welcome to Modrinth
						<img :src="frog" alt="" class="h-8 w-8 [image-rendering:pixelated]" />
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
		<div class="flex flex-col items-center justify-end gap-4 text-sm leading-5 text-secondary">
			<span class="whitespace-nowrap">Escaping another launcher?</span>
			<ButtonStyled>
				<button
					class="!h-10 !gap-2 !rounded-[0.875rem] !px-4 !py-2.5 !font-medium"
					:disabled="offline"
					@click="showImportModal?.()"
				>
					<ImportIcon />
					Import from launcher
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<style scoped>
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

.welcome-icon {
	box-shadow:
		0 0 0 1.5px #07180d,
		0 1.5rem 3rem rgb(0 0 0 / 3%),
		0 0.625rem 1.125rem rgb(0 0 0 / 3%),
		0 0.3125rem 0.5rem rgb(0 0 0 / 4%),
		0 0.125rem 0.25rem rgb(0 0 0 / 4%),
		0 0.625rem 13.75rem color-mix(in srgb, var(--color-brand) 32%, transparent);
}
</style>
