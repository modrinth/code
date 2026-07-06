<template>
	<PageHeader
		:title="project.title"
		:summary="project.description"
		@contextmenu="emit('contextmenu', $event)"
	>
		<template #leading>
			<Avatar :src="project.icon_url" :alt="project.title" :tint-by="project.id" size="96px" />
		</template>

		<template v-if="showStatusBadge" #badges>
			<ProjectStatusBadge :status="project.status" />
		</template>

		<template #metadata>
			<PageHeaderMetadata>
				<template v-if="isServerProject">
					<ServerDetails
						v-if="projectV3?.status !== 'draft'"
						:online-players="projectV3?.minecraft_java_server?.ping?.data?.players_online ?? 0"
						:status-online="!!projectV3?.minecraft_java_server?.ping?.data"
						:recent-plays="projectV3?.minecraft_java_server?.verified_plays_2w ?? 0"
					/>
				</template>
				<template v-else>
					<PageHeaderMetadataNumberItem
						:icon="DownloadIcon"
						:value="project.downloads"
						:label="formatMessage(messages.downloadsStat, { count: project.downloads })"
						:tooltip="formatNumber(project.downloads)"
						class="cursor-help"
					/>
					<PageHeaderMetadataNumberItem
						:icon="HeartIcon"
						:value="project.followers"
						:label="formatMessage(messages.followersStat, { count: project.followers })"
						:tooltip="formatNumber(project.followers)"
						class="cursor-help"
					/>
				</template>
				<PageHeaderMetadataTagsItem v-if="project.categories.length > 0" class="hidden md:flex">
					<TagItem
						v-for="category in project.categories"
						:key="category"
						:action="() => emit('category', category)"
					>
						<FormattedTag :tag="category" />
					</TagItem>
				</PageHeaderMetadataTagsItem>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<slot name="actions" />
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, HeartIcon } from '@modrinth/assets'

import { defineMessages, useFormatNumber, useVIntl } from '../../composables'
import Avatar from '../base/Avatar.vue'
import FormattedTag from '../base/FormattedTag.vue'
import PageHeader from '../base/page-header/index.vue'
import PageHeaderMetadata from '../base/page-header/metadata/index.vue'
import PageHeaderMetadataNumberItem from '../base/page-header/metadata/page-header-metadata-number-item.vue'
import PageHeaderMetadataTagsItem from '../base/page-header/metadata/page-header-metadata-tags-item.vue'
import PageHeaderActions from '../base/page-header/page-header-actions.vue'
import TagItem from '../base/TagItem.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'
import ServerDetails from './server/ServerDetails.vue'

type HeaderProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'title' | 'description' | 'status' | 'downloads' | 'followers' | 'categories'
> & {
	icon_url?: string | null
}

type HeaderProjectV3 = Pick<Labrinth.Projects.v3.Project, 'status' | 'minecraft_java_server'>

withDefaults(
	defineProps<{
		project: HeaderProject
		projectV3?: HeaderProjectV3 | null
		isServerProject?: boolean
		showStatusBadge?: boolean
	}>(),
	{
		projectV3: null,
		isServerProject: false,
		showStatusBadge: false,
	},
)

const emit = defineEmits<{
	category: [category: string]
	contextmenu: [event: MouseEvent]
}>()

const messages = defineMessages({
	downloadsStat: {
		id: 'project.stats.downloads-label',
		defaultMessage: '{count, plural, one {download} other {downloads}}',
	},
	followersStat: {
		id: 'project.stats.followers-label',
		defaultMessage: '{count, plural, one {follower} other {followers}}',
	},
})

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
</script>
