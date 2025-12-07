<script setup lang="ts">
import {
	IconSelect,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	SettingsLabel,
	UnsavedChangesPopup,
	useSavable,
} from '@modrinth/ui'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const { projectV2: project, refreshProject } = injectProjectPageContext()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()

const saving = ref(false)

const { saved, current, reset, save } = useSavable(
	() => ({
		title: project.value.title,
		tagline: project.value.description,
		url: project.value.slug,
		icon: project.value.icon_url,
	}),
	({ title, tagline, url }) => {
		const data: Record<string, string> = {
			...(title !== undefined && { title }),
			...(tagline !== undefined && { description: tagline }),
			...(url !== undefined && { slug: url }),
		}

		if (data) {
			saving.value = true
			client.labrinth.projects_v2
				.edit(project.value.id, { title, description: tagline, slug: url })
				.then(() => refreshProject().then(reset))
				.catch(handleError)
				.finally(() => (saving.value = false))
		}
	},
)

const messages = defineMessages({
	nameTitle: {
		id: 'project.settings.general.name.title',
		defaultMessage: 'Name',
	},
	nameDescription: {
		id: 'project.settings.general.name.description',
		defaultMessage:
			"Avoid prefixes, suffixes, parentheticals, or added descriptions—just the project's actual name.",
	},
	taglineTitle: {
		id: 'project.settings.general.tagline.title',
		defaultMessage: 'Tagline',
	},
	taglineDescription: {
		id: 'project.settings.general.tagline.description',
		defaultMessage: 'Summarize your project in no more than one sentence.',
	},
	urlTitle: {
		id: 'project.settings.general.url.title',
		defaultMessage: 'URL',
	},
})

const placeholders: { name: MessageDescriptor; tagline: MessageDescriptor }[] = [
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.1',
			defaultMessage: 'e.g. Nether Overhaul 2',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.1',
			defaultMessage: 'e.g. Overhauls game progression to revolve around the Nether.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.2',
			defaultMessage: 'e.g. Construction Equipment',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.2',
			defaultMessage: 'e.g. Adds wearable construction gear.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.3',
			defaultMessage: 'e.g. Better than Caving',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.3',
			defaultMessage: 'e.g. Adds realistic mineshaft-building mechanics.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.4',
			defaultMessage: 'e.g. Enhanced Portals',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.4',
			defaultMessage: 'e.g. Improves how Nether portals link to each other.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.5',
			defaultMessage: 'e.g. Dangerous Mobs',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.5',
			defaultMessage:
				'e.g. Adds powerful boss versions of the normal mobs to encounter in the night.',
		},
	}),
]

const placeholderIndex = useState<number>('project-settings-random-placeholder', () =>
	Math.floor(Math.random() * (placeholders.length + 1)),
)

const placeholder = computed(() => placeholders[placeholderIndex.value] ?? placeholders[0])
</script>
<template>
	<div>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			@reset="reset"
			@save="save"
		/>
		<div class="base-card block">
			<div class="group relative float-end ml-4">
				<IconSelect v-model="current.icon" />
			</div>
			<div>
				<SettingsLabel
					id="project-name"
					:title="messages.nameTitle"
					:description="messages.nameDescription"
				/>
				<div class="flex">
					<input
						id="project-name"
						v-model="current.title"
						:placeholder="formatMessage(placeholder.name)"
						autocomplete="off"
						maxlength="50"
						class="flex-grow"
						type="text"
					/>
				</div>
			</div>
			<div class="mt-4">
				<SettingsLabel
					id="project-tagline"
					:title="messages.taglineTitle"
					:description="messages.taglineDescription"
				/>
				<input
					id="project-tagline"
					v-model="current.tagline"
					:placeholder="formatMessage(placeholder.tagline)"
					autocomplete="off"
					maxlength="120"
					class="w-full"
					type="text"
				/>
			</div>
			<div class="mt-4">
				<SettingsLabel id="project-url" :title="messages.urlTitle" />
				<div class="text-input-wrapper">
					<div class="text-input-wrapper__before">https://modrinth.com/project/</div>
					<input
						id="project-url"
						v-model="current.url"
						type="text"
						maxlength="64"
						autocomplete="off"
					/>
				</div>
			</div>
		</div>
	</div>
</template>
