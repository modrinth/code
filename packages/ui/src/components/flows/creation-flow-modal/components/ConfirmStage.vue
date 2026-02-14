<template>
	<div class="space-y-6">
		<div class="flex flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
			<span class="font-semibold text-contrast">Summary</span>
			<div class="flex flex-col gap-1 text-sm">
				<div v-if="ctx.worldType.value === 'modpack' && ctx.modpackSelection.value">
					<span class="text-secondary">Modpack:</span>
					{{ ctx.modpackSelection.value.name }}
				</div>
				<div v-else-if="ctx.worldType.value === 'modpack' && ctx.modpackFile.value">
					<span class="text-secondary">Modpack file:</span>
					{{ ctx.modpackFile.value.name }}
				</div>
				<template v-else>
					<div v-if="ctx.selectedLoader.value">
						<span class="text-secondary">Loader:</span>
						{{ formatLoader(ctx.selectedLoader.value) }}
					</div>
					<div>
						<span class="text-secondary">Game version:</span>
						{{ ctx.selectedGameVersion.value }}
					</div>
					<div v-if="ctx.selectedLoaderVersion.value && !ctx.hideLoaderFields.value">
						<span class="text-secondary">Loader version:</span>
						{{ ctx.selectedLoaderVersion.value }}
					</div>
				</template>
			</div>
		</div>

		<div v-if="!ctx.isInitialSetup" class="flex flex-col gap-4">
			<div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
				<div class="flex w-full flex-row items-center justify-between">
					<label class="w-full text-lg font-bold text-contrast" for="hard-reset">
						Erase all data
					</label>
					<Toggle id="hard-reset" v-model="ctx.hardReset.value" class="shrink-0" />
				</div>
				<div class="text-sm">
					Removes all data on your server, including your worlds, mods, and configuration files,
					then reinstalls it with the selected version.
				</div>
				<div class="text-sm font-bold">
					This does not affect your backups, which are stored off-site.
				</div>
			</div>

			<BackupWarning backup-link="" />
		</div>
	</div>
</template>

<script setup lang="ts">
import Toggle from '../../../base/Toggle.vue'
import BackupWarning from '../../../servers/backups/BackupWarning.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()

const loaderDisplayNames: Record<string, string> = {
	fabric: 'Fabric',
	neoforge: 'NeoForge',
	forge: 'Forge',
	quilt: 'Quilt',
	paper: 'Paper',
	purpur: 'Purpur',
	vanilla: 'Vanilla',
}

const formatLoader = (loader: string) =>
	loaderDisplayNames[loader] ?? loader.charAt(0).toUpperCase() + loader.slice(1)
</script>
