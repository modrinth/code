<template>
	<IconButton
		v-tooltip="formatMessage(commonMessages.settingsLabel)"
		type="quiet"
		size="xs"
		:label="formatMessage(commonMessages.settingsLabel)"
		:aria-pressed="workbench.isOpen(SETTINGS_PANEL_ID)"
		@click="toggle"
	>
		<SettingsIcon aria-hidden="true" />
	</IconButton>
</template>

<script setup lang="ts">
import { SettingsIcon } from '@modrinth/assets'

import { IconButton } from '#ui/components/base/buttons'
import { useVIntl } from '#ui/composables/i18n'
import { injectWorkbench } from '#ui/layouts/shared/workbench/providers/workbench'
import { commonMessages } from '#ui/utils/common-messages'

import { SETTINGS_PANEL_ID } from '../settings'

/**
 * Brings the settings panel into view (opening it if closed), wherever it was moved to, and
 * closes it again when it's already in view.
 */
const { formatMessage } = useVIntl()
const workbench = injectWorkbench()

function toggle() {
	if (workbench.isShown(SETTINGS_PANEL_ID)) workbench.closePanel(SETTINGS_PANEL_ID)
	else workbench.revealPanel(SETTINGS_PANEL_ID)
}
</script>
