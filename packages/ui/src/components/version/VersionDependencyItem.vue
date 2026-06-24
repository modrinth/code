<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n.ts'

import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import type { DependencyContext } from './index.ts'

const { formatMessage } = useVIntl()

withDefaults(
	defineProps<{
		context: DependencyContext
		dependencyLink?: string
		linkTabbable?: boolean
	}>(),
	{
		dependencyLink: undefined,
		linkTabbable: false,
	},
)

const messages = defineMessages({
	anyVersion: {
		id: 'version.section.dependencies.any-version',
		defaultMessage: 'Any version',
	},
	anyCompatibleVersion: {
		id: 'version.section.dependencies.any-compatible-version',
		defaultMessage: 'Any compatible version',
	},
	unavailableVersion: {
		id: 'version.section.dependencies.unavailable-version',
		defaultMessage: 'Unavailable version',
	},
	unavailableVersionDescription: {
		id: 'version.section.dependencies.unavailable-version.description',
		defaultMessage: 'This version may have been deleted or is being reviewed by moderators.',
	},
	unavailableRequiredVersionDescription: {
		id: 'version.section.dependencies.unavailable-version.description.required',
		defaultMessage:
			'The required version may have been deleted or is being reviewed by moderators.',
	},
})
</script>
<template>
	<div class="flex gap-3 items-center">
		<AutoLink
			:to="dependencyLink"
			class="flex gap-3 items-center group"
			:tabindex="linkTabbable ? undefined : '-1'"
		>
			<Avatar :src="context.project?.icon_url" alt="" size="48px" no-shadow />
			<div class="flex flex-col gap-1">
				<span
					class="text-contrast font-medium"
					:class="{ 'group-hover:underline': !!dependencyLink }"
					>{{ context.project?.title ?? context.dependency.file_name }}</span
				>
				<span class="text-secondary flex items-center gap-1">
					<template v-if="!context.dependency.version_id">
						{{
							formatMessage(
								context.dependency.dependency_type === 'incompatible'
									? messages.anyVersion
									: messages.anyCompatibleVersion,
							)
						}}
					</template>
					<template v-else-if="context.version">
						{{ context.version.version_number }}
					</template>
					<template v-else>
						{{ formatMessage(messages.unavailableVersion) }}
						<InfoIcon
							v-tooltip="
								formatMessage(
									context.dependency.dependency_type === 'required'
										? messages.unavailableRequiredVersionDescription
										: messages.unavailableVersionDescription,
								)
							"
							class="shrink-0"
						/>
					</template>
				</span>
			</div>
		</AutoLink>
		<div class="flex items-center gap-2 ml-auto">
			<slot />
		</div>
	</div>
</template>
