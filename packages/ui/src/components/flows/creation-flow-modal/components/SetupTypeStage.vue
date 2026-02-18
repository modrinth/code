<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">
			{{ ctx.flowType === 'instance' ? 'Choose instance type' : 'Select world type' }}
		</span>

		<!-- Instance flow options -->
		<template v-if="ctx.flowType === 'instance'">
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="BoxesIcon"
					title="Custom setup"
					description="Start from scratch by picking a loader and game version."
					:selected="ctx.initialSetupType === 'custom'"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="PackageIcon"
					title="Modpack base"
					description="Use a popular Modpack as your starting point."
					:selected="ctx.initialSetupType === 'modpack'"
					@click="setSetupType('modpack')"
				/>
				<BigOptionButton
					:icon="BoxImportIcon"
					title="Import Instance"
					description="Import an instance from Prism, CurseForge, or similar."
					@click="ctx.setImportMode()"
				/>
			</div>
			<span class="text-sm text-secondary">
				An instance is a Minecraft setup with a specific loader, version, and mods.
			</span>
		</template>

		<!-- World / Server onboarding flow options -->
		<template v-else>
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="PackageIcon"
					title="Modpack base"
					description="Use a popular Modpack as your starting point."
					:selected="ctx.initialSetupType === 'modpack'"
					@click="setSetupType('modpack')"
				/>
				<BigOptionButton
					:icon="BoxesIcon"
					title="Custom setup"
					description="Start from scratch by picking a loader and game version."
					:selected="ctx.initialSetupType === 'custom'"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="BoxIcon"
					title="Vanilla Minecraft"
					description="Classic Minecraft with no mods or plugins."
					:selected="ctx.initialSetupType === 'vanilla'"
					@click="setSetupType('vanilla')"
				/>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { BoxesIcon, BoxIcon, BoxImportIcon, PackageIcon } from '@modrinth/assets'

import BigOptionButton from '../../../base/BigOptionButton.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const { setSetupType } = ctx
</script>
