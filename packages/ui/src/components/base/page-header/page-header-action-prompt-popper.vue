<template>
	<div class="grid grid-cols-[min-content] gap-1">
		<div class="flex min-w-48 items-center justify-between gap-8">
			<h3 class="m-0 flex items-center gap-2 whitespace-nowrap text-base font-bold text-contrast">
				{{ prompt.title }}
				<span
					v-if="prompt.badge"
					class="inline-flex items-center rounded-full border border-solid border-brand-highlight bg-brand-highlight px-2 py-1 text-sm font-semibold leading-none text-brand"
				>
					{{ prompt.badge }}
				</span>
			</h3>
			<ButtonStyled size="small" circular>
				<button v-tooltip="prompt.dismissLabel" type="button" @click="prompt.onDismiss?.()">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
		<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
			{{ prompt.description }}
		</p>
		<p v-if="prompt.footer" class="m-0 text-wrap text-sm font-bold text-primary">
			{{ prompt.footer }}
		</p>
	</div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'

import ButtonStyled from '../ButtonStyled.vue'
import type { PageHeaderActionPrompt } from './types'

defineProps<{
	prompt: PageHeaderActionPrompt
}>()
</script>
