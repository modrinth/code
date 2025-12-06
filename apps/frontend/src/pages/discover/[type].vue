<script setup lang="ts">
import { commonProjectTypeCategoryMessages } from '@modrinth/ui'
import { useVIntl } from '@vintl/vintl'

const route = useRoute()
const { formatMessage } = useVIntl()

if (!route.params.type || typeof route.params.type !== 'string') {
	throw createError({
		statusCode: 404,
	})
}

const messages = defineMessages({
	discover: {
		id: 'discover.title',
		defaultMessage: 'Discover',
	},
})

function isProjectTypeKey(value: string): value is keyof typeof commonProjectTypeCategoryMessages {
	return value in commonProjectTypeCategoryMessages
}

const type = route.params.type.replaceAll(/^\/|s\/?$/g, '')
const titleMessage = isProjectTypeKey(type)
	? commonProjectTypeCategoryMessages[type]
	: messages.discover
</script>
<template>
	<Head>
		<Title>{{ formatMessage(titleMessage) }} - Modrinth</Title>
	</Head>
	<NuxtPage :type="type" />
</template>
