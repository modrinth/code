<script setup lang="ts">
import { defineMessages, injectAuth, useVIntl } from '@modrinth/ui'

import { useHiddenLibraryInstances } from '@/features/library/hidden-instances/use-hidden-library-instances'
import { isUninstalledSharedInstance } from '@/features/library/library-instance'
import { useSharedInstanceOwners } from '@/features/library/shared-instances/use-shared-instance-owners'

import HiddenInstalledInstanceRow from './installed-row.vue'
import HiddenSharedInstanceRow from './shared-row.vue'

const { formatMessage } = useVIntl()
const auth = injectAuth()

const messages = defineMessages({
	title: {
		id: 'app.behavior-settings.hidden-instances.title',
		defaultMessage: 'Hidden instances',
	},
	description: {
		id: 'app.behavior-settings.hidden-instances.description',
		defaultMessage:
			"Instances you've hidden from your library on this device. Hidden instances aren't synced with your Modrinth account.",
	},
	empty: {
		id: 'app.behavior-settings.hidden-instances.empty',
		defaultMessage: "You haven't hidden any instances yet.",
	},
})

const hiddenInstances = useHiddenLibraryInstances()
const { getSharedInstanceOwner } = useSharedInstanceOwners(hiddenInstances)
</script>

<template>
	<section v-if="auth.user.value" class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.title) }}
		</h2>
		<p class="m-0 mt-1">{{ formatMessage(messages.description) }}</p>
		<div v-if="hiddenInstances.length > 0" class="mt-4 flex flex-col gap-2">
			<template v-for="instance in hiddenInstances" :key="instance.id">
				<HiddenSharedInstanceRow
					v-if="isUninstalledSharedInstance(instance)"
					:instance="instance"
					:owner="getSharedInstanceOwner(instance)"
				/>
				<HiddenInstalledInstanceRow v-else :instance="instance" />
			</template>
		</div>
		<p v-else class="m-0 mx-auto mt-6 text-center text-sm text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
	</section>
</template>
