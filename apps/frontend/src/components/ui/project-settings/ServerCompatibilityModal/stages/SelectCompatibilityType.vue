<template>
	<div class="flex flex-col gap-6">
		<DataLossWarningBanner />
		<div class="flex flex-col gap-4">
			<div class="font-semibold text-contrast">{{ formatMessage(messages.selectType) }}</div>
			<div class="flex w-full flex-col gap-2">
				<button
					v-for="option in options"
					:key="option.value"
					class="flex !w-full items-center gap-4 rounded-3xl bg-surface-4 p-3 text-left transition-all hover:brightness-125"
					@click="selectType(option.value)"
				>
					<div
						class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl border border-solid border-surface-5"
					>
						<component :is="option.icon" class="h-9 w-9 text-secondary" />
					</div>
					<div class="flex flex-col gap-0.5">
						<span class="font-bold text-contrast">{{ formatMessage(option.label) }}</span>
						<span class="text-sm text-secondary">{{ formatMessage(option.description) }}</span>
					</div>
				</button>
			</div>
			<div class="text-sm text-secondary">
				{{ formatMessage(messages.customModpackNotice) }}
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { BoxIcon, PackageIcon, PackagePlusIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'

import {
	type CompatibilityType,
	injectServerCompatibilityContext,
} from '~/providers/manage-server-compatibility-modal'

import DataLossWarningBanner from '../DataLossWarningBanner.vue'
import { compatibilityMessages as messages } from '../messages'

const { formatMessage } = useVIntl()

const ctx = injectServerCompatibilityContext()

const options = [
	{
		value: 'vanilla' as CompatibilityType,
		label: messages.vanillaServer,
		description: messages.vanillaDescription,
		icon: BoxIcon,
	},
	{
		value: 'published-modpack' as CompatibilityType,
		label: messages.publishedModpack,
		description: messages.publishedModpackDescription,
		icon: PackageIcon,
	},
	{
		value: 'custom-modpack' as CompatibilityType,
		label: messages.customModpack,
		description: messages.customModpackDescription,
		icon: PackagePlusIcon,
	},
]

function selectType(type: CompatibilityType) {
	ctx.compatibilityType.value = type
	ctx.modal.value?.nextStage()
}
</script>
