<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">Select world type</span>
		<div class="flex flex-col gap-3">
			<BigOptionButton
				:icon="PackageIcon"
				title="Modpack base"
				description="Use a popular Modpack as your starting point."
				@click="setWorldType('modpack')"
			/>
			<BigOptionButton
				:icon="BoxesIcon"
				title="Custom setup"
				description="Pick a loader and game version."
				@click="setWorldType('custom')"
			/>
			<BigOptionButton
				:icon="BoxIcon"
				title="Vanilla Minecraft"
				description="Classic Minecraft with no mods or plugins."
				@click="setWorldType('vanilla')"
			/>
		</div>
		<span class="text-sm text-secondary">
			We'll take care of installing the right loader to your server!
		</span>
	</div>
</template>

<script setup lang="ts">
import { BoxIcon, BoxesIcon, PackageIcon } from '@modrinth/assets'

import BigOptionButton from '../../../../base/BigOptionButton.vue'
import { injectCreateWorldContext } from '../create-world-context'

const { setWorldType } = injectCreateWorldContext()
</script>
