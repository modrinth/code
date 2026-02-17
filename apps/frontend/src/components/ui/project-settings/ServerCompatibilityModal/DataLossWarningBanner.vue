<template>
	<Admonition v-if="showDataLossWarning" type="critical" header="Data loss warning">
		Changing the compatibility type will reset your previous compatibility settings and redistribute
		the new settings to users in the Modrinth App.
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition } from '@modrinth/ui'

import { injectServerCompatibilityContext } from './manage-server-compatibility-modal'

const { showDataLossWarning } = injectServerCompatibilityContext()
</script>
