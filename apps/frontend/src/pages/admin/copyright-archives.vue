<script setup lang="ts">
import { isAdmin } from '@modrinth/utils'

import CopyrightArchives from '~/components/copyright-archives/index.vue'

definePageMeta({
	middleware: [
		'auth',
		async () => {
			const auth = await useAuth()
			if (!auth.value.user || !isAdmin(auth.value.user)) {
				throw createError({ fatal: true, statusCode: 401, statusMessage: 'Unauthorized' })
			}
		},
	],
})
</script>

<template>
	<CopyrightArchives />
</template>
