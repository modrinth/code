<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">Select world type</span>
		<div class="flex flex-col gap-3">
			<BigOptionButton
				:icon="PackageIcon"
				title="Modpack base"
				description="Use a popular Modpack as your starting point."
				:selected="ctx.initialWorldType === 'modpack'"
				@click="setWorldType('modpack')"
			/>
			<BigOptionButton
				:icon="BoxesIcon"
				title="Custom setup"
				description="Start from scratch by picking a loader and game version."
				:selected="ctx.initialWorldType === 'custom'"
				@click="setWorldType('custom')"
			/>
			<BigOptionButton
				:icon="BoxIcon"
				title="Vanilla Minecraft"
				description="Classic Minecraft with no mods or plugins."
				:selected="ctx.initialWorldType === 'vanilla'"
				@click="setWorldType('vanilla')"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { BoxesIcon, BoxIcon, PackageIcon } from '@modrinth/assets'

import BigOptionButton from '../../../base/BigOptionButton.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const { setWorldType } = ctx
</script>
