<template>
	<ServersManageWorldRootLayout>
		<RouterView v-slot="{ Component }">
			<template v-if="Component">
				<Suspense>
					<component :is="Component" />
				</Suspense>
			</template>
		</RouterView>
	</ServersManageWorldRootLayout>
</template>

<script setup lang="ts">
import { ServersManageWorldRootLayout } from '@modrinth/ui'
import { RouterView } from 'vue-router'
</script>
