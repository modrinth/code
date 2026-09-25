<script setup lang="ts">
import { renderString } from '@modrinth/utils'
import { Heading } from '@vue-email/components'

import StyledEmail from '../shared/StyledEmail.vue'

const props = defineProps<{
	title: string
	body: string
}>()
</script>

<template>
	<StyledEmail :title="props.title">
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			{{ props.title }}
		</Heading>

		<div v-html="renderString(props.body)" />
	</StyledEmail>
</template>
