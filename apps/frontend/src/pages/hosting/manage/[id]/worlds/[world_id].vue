<script setup lang="ts">
import { ServersManageWorldRootLayout } from '@modrinth/ui'

const route = useNativeRoute()
</script>

<template>
	<ServersManageWorldRootLayout>
		<NuxtPage :route="route" />
	</ServersManageWorldRootLayout>
</template>
