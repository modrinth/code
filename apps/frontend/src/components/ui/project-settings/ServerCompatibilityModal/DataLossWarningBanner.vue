<template>
	<Admonition
		v-if="isSwitchingCompatibilityType"
		type="critical"
		:header="formatMessage(messages.dataLossWarning)"
	>
		{{ formatMessage(messages.dataLossDescription) }}
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import { injectServerCompatibilityContext } from '../../../../providers/manage-server-compatibility-modal'
import { compatibilityMessages as messages } from './messages'

const { formatMessage } = useVIntl()

const { isSwitchingCompatibilityType } = injectServerCompatibilityContext()
</script>
