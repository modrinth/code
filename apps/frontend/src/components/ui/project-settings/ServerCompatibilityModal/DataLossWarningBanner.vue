<template>
	<Admonition v-if="isSwitchingCompatibilityType" type="critical" header="Data loss warning">
		Changing the compatibility type will reset your previous compatibility settings and redistribute
		the new settings to users in the Modrinth App.
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition } from '@modrinth/ui'

import { injectServerCompatibilityContext } from '../../../../providers/manage-server-compatibility-modal'

const { isSwitchingCompatibilityType } = injectServerCompatibilityContext()
</script>
