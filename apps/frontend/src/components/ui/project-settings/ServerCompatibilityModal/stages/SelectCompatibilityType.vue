<template>
	<div class="flex flex-col gap-6">
		<DataLossWarningBanner />
		<div class="flex flex-col gap-4">
			<div class="font-semibold text-contrast">Select compatibility type</div>
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
						<span class="font-bold text-contrast">{{ option.label }}</span>
						<span class="text-sm text-secondary">{{ option.description }}</span>
					</div>
				</button>
			</div>
			<div class="text-sm text-secondary">
				Servers with custom modpacks should not be uploaded as separate modpack projects.
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { BoxIcon, PackageIcon, PackageOpenIcon } from '@modrinth/assets'

import {
	type CompatibilityType,
	injectServerCompatibilityContext,
} from '~/providers/manage-server-compatibility-modal'
import DataLossWarningBanner from '../DataLossWarningBanner.vue'

const ctx = injectServerCompatibilityContext()

const options = [
	{
		value: 'vanilla' as CompatibilityType,
		label: 'Vanilla server',
		description: 'A vanilla server with no mods.',
		icon: BoxIcon,
	},
	{
		value: 'published-modpack' as CompatibilityType,
		label: 'Published modpack',
		description: 'A modded server using a published modpack.',
		icon: PackageIcon,
	},
	{
		value: 'custom-modpack' as CompatibilityType,
		label: 'Custom modpack',
		description: 'A modded server using a custom modpack.',
		icon: PackageOpenIcon,
	},
]

function selectType(type: CompatibilityType) {
	ctx.compatibilityType.value = type
	ctx.modal.value?.nextStage()
}
</script>
