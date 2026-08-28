<script setup lang="ts">
import { injectPageContext } from '@modrinth/ui'

const { hierarchicalSidebarAvailable } = injectPageContext()
</script>
<template>
	<Teleport v-if="hierarchicalSidebarAvailable" to="#sidebar">
		<aside id="page-sidebar">
			<slot />
		</aside>
	</Teleport>
	<aside v-else id="page-sidebar">
		<slot />
	</aside>
</template>
