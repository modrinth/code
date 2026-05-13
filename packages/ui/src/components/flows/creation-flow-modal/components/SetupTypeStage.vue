<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">
			{{ setupTypeTitle }}
		</span>

		<!-- Instance flow options -->
		<template v-if="ctx.flowType === 'instance'">
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="BoxesIcon"
					:title="formatMessage(messages.customSetupTitle)"
					:description="formatMessage(messages.customSetupDescription)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="PackageIcon"
					:title="formatMessage(messages.modpackBaseTitle)"
					:description="formatMessage(messages.modpackBaseDescription)"
					@click="setSetupType('modpack')"
				/>
				<BigOptionButton
					:icon="BoxImportIcon"
					:title="formatMessage(messages.importInstanceTitle)"
					:description="formatMessage(messages.importInstanceDescription)"
					@click="ctx.setImportMode()"
				/>
			</div>
			<span class="text-sm text-secondary">
				{{ formatMessage(messages.instanceDescription) }}
			</span>
		</template>

		<!-- World / Server onboarding flow options -->
		<template v-else>
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="PackageIcon"
					:title="formatMessage(messages.modpackBaseTitle)"
					:description="formatMessage(messages.modpackBaseDescription)"
					@click="setSetupType('modpack')"
				/>
				<BigOptionButton
					:icon="BoxesIcon"
					:title="formatMessage(messages.customSetupTitle)"
					:description="formatMessage(messages.customSetupDescription)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="BoxIcon"
					:title="formatMessage(messages.vanillaMinecraftTitle)"
					:description="formatMessage(messages.vanillaMinecraftDescription)"
					@click="setSetupType('vanilla')"
				/>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { BoxesIcon, BoxIcon, BoxImportIcon, PackageIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import BigOptionButton from '../../../base/BigOptionButton.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const debug = useDebugLogger('SetupTypeStage')
const ctx = injectCreationFlowContext()
const { setSetupType: _setSetupType } = ctx
const { formatMessage } = useVIntl()

const messages = defineMessages({
	instanceTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.instance',
		defaultMessage: 'Choose instance type',
	},
	installationTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.installation',
		defaultMessage: 'Select installation type',
	},
	worldTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.world',
		defaultMessage: 'Select world type',
	},
	customSetupTitle: {
		id: 'creation-flow.modal.setup-type.option.custom-setup.title',
		defaultMessage: 'Custom setup',
	},
	customSetupDescription: {
		id: 'creation-flow.modal.setup-type.option.custom-setup.description',
		defaultMessage: 'Start from scratch by picking a loader and game version.',
	},
	modpackBaseTitle: {
		id: 'creation-flow.modal.setup-type.option.modpack-base.title',
		defaultMessage: 'Install modpack',
	},
	modpackBaseDescription: {
		id: 'creation-flow.modal.setup-type.option.modpack-base.description',
		defaultMessage: 'Browse modpacks on Modrinth or import one from a file.',
	},
	importInstanceTitle: {
		id: 'creation-flow.modal.setup-type.option.import-instance.title',
		defaultMessage: 'Import instance',
	},
	importInstanceDescription: {
		id: 'creation-flow.modal.setup-type.option.import-instance.description',
		defaultMessage: 'Import an instance from Prism, CurseForge, or similar.',
	},
	instanceDescription: {
		id: 'creation-flow.modal.setup-type.instance.description',
		defaultMessage: 'An instance is a Minecraft setup with a specific loader, version, and mods.',
	},
	vanillaMinecraftTitle: {
		id: 'creation-flow.modal.setup-type.option.vanilla-minecraft.title',
		defaultMessage: 'Vanilla Minecraft',
	},
	vanillaMinecraftDescription: {
		id: 'creation-flow.modal.setup-type.option.vanilla-minecraft.description',
		defaultMessage: 'Classic Minecraft with no mods or plugins.',
	},
})

const setupTypeTitle = computed(() => {
	if (ctx.flowType === 'instance') {
		return formatMessage(messages.instanceTypeTitle)
	}
	if (ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server') {
		return formatMessage(messages.installationTypeTitle)
	}
	return formatMessage(messages.worldTypeTitle)
})

function setSetupType(type: 'modpack' | 'custom' | 'vanilla') {
	debug('selected:', type)
	_setSetupType(type)
}
</script>
