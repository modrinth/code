<script setup lang="ts">
import { defineMessages, type SortField, SortMenu, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectLibraryView } from '../../library-context'
import { type LibrarySort, librarySortKinds, librarySortOptions } from '../../types'

const { sortMode } = injectLibraryView()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	name: { id: 'app.library.sort.name', defaultMessage: 'Name' },
	loader: { id: 'app.library.sort.loader', defaultMessage: 'Loader' },
	gameVersion: { id: 'app.library.sort.game-version', defaultMessage: 'Game version' },
	lastPlayed: { id: 'app.library.sort.last-played', defaultMessage: 'Last played' },
	hoursPlayed: { id: 'app.library.sort.hours-played', defaultMessage: 'Hours played' },
	dateCreated: { id: 'app.library.sort.date-created', defaultMessage: 'Date created' },
	dateModified: { id: 'app.library.sort.date-modified', defaultMessage: 'Date modified' },
})

const sortLabels = {
	Name: messages.name,
	'Last played': messages.lastPlayed,
	'Hours played': messages.hoursPlayed,
	'Date created': messages.dateCreated,
	'Date modified': messages.dateModified,
	Loader: messages.loader,
	'Game version': messages.gameVersion,
}

const sortFields = computed<SortField<LibrarySort>[]>(() =>
	librarySortOptions.map((option) => ({
		value: option,
		label: formatMessage(sortLabels[option]),
		kind: librarySortKinds[option],
	})),
)
</script>

<template>
	<SortMenu v-model="sortMode" :fields="sortFields" />
</template>
