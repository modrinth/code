<template>
	<ServersManageInstanceRootLayout>
		<RouterView v-slot="{ Component }">
			<template v-if="Component">
				<Suspense>
					<component :is="Component" />
				</Suspense>
			</template>
		</RouterView>
	</ServersManageInstanceRootLayout>
</template>

<script setup lang="ts">
import { ServersManageInstanceRootLayout } from '@modrinth/ui'
import { RouterView } from 'vue-router'
</script>
