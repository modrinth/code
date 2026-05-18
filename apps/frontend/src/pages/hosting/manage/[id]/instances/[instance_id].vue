<script setup lang="ts">
import { ServersManageInstanceRootLayout } from '@modrinth/ui'

const route = useNativeRoute()
</script>

<template>
	<ServersManageInstanceRootLayout>
		<NuxtPage :route="route" />
	</ServersManageInstanceRootLayout>
</template>
