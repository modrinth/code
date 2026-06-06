<script setup lang="ts">
import { RightArrowIcon, SearchIcon, SortAscIcon, SortDescIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	EmptyState,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import ExternalProjectPermissionsCard from '@modrinth/ui/src/components/external_files/ExternalProjectPermissionsCard.vue'
import { ref } from 'vue'

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()

if (!flags.value.modpackPermissionsPage) {
	throw createError({
		fatal: true,
		statusCode: 404,
	})
}

const externalFiles = ref([{}])
const searchQuery = ref('')
const currentSortType = ref('Oldest')

const sortTypes: ComboboxOption<string>[] = [
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
]
const messages = defineMessages({
	searchPlaceholder: {
		id: 'project.settings.permissions.search-placeholder',
		defaultMessage:
			'Search {count} {count, plural, one {external project} other {external projects}}...',
	},
	infoBannerTitle: {
		id: 'project.settings.permissions.info-banner.title',
		defaultMessage: 'Learn how attributions work',
	},
	infoBannerDescription: {
		id: 'project.settings.permissions.info-banner.description',
		defaultMessage: `If you include content that isn’t hosted on Modrinth, you need to let us know where it’s from and verify that you have permission to distribute the files. Check out <link>our guide</link> to learn about how to do this properly!`,
	},
	learnMore: {
		id: 'project.settings.permissions.learn-more',
		defaultMessage: 'Learn more',
	},
	emptyStateHeading: {
		id: 'project.settings.permissions.empty-state.heading',
		defaultMessage: `You're all set!`,
	},
	emptyStateDescription: {
		id: 'project.settings.permissions.empty-state.description',
		defaultMessage: `None of your versions contain external content, so you don't need to worry about obtaining permissions.`,
	},
	completedTitle: {
		id: 'project.settings.permissions.completed.title',
		defaultMessage: `Attributions completed!`,
	},
	completedDescription: {
		id: 'project.settings.permissions.completed.description',
		defaultMessage: 'All external content has attributions provided.',
	},
	failTitle: {
		id: 'project.settings.permissions.fail.title',
		defaultMessage: `Some content can't be included`,
	},
	failDescription: {
		id: 'project.settings.permissions.fail.description',
		defaultMessage: `You don't have permission to redistribute some of the external content you've added. In order to publish on Modrinth, remove the infringing content.`,
	},
	attentionNeededTitle: {
		id: 'project.settings.permissions.attention-needed.title',
		defaultMessage: `Unknown embedded content`,
	},
	attentionNeededDescriptionApproved: {
		id: 'project.settings.permissions.attention-needed.description.proj-approved',
		defaultMessage: `Please provide proof that you have permission to redistribute all of the following files and any withheld versions will be automatically published.`,
	},
	attentionNeededDescriptionDraft: {
		id: 'project.settings.permissions.attention-needed.description.proj-draft',
		defaultMessage: `Please provide proof that you have permission to redistribute all of the following files before you can submit your project for review.`,
	},
})

function dismissInfoBanner() {
	flags.value.dismissedExternalProjectsInfo = true
	saveFeatureFlags()
}
</script>
<template>
	<template v-if="externalFiles.length > 0">
		<Admonition
			v-if="!flags.dismissedExternalProjectsInfo"
			type="info"
			class="mb-4"
			:header="formatMessage(messages.infoBannerTitle)"
			dismissible
			@dismiss="dismissInfoBanner"
		>
			<IntlFormatted :message-id="messages.infoBannerDescription">
				<template #link="{ children }">
					<a class="text-link" target="_blank"> <component :is="() => children" /> </a>
				</template>
			</IntlFormatted>
			<template #actions>
				<div class="flex">
					<ButtonStyled color="blue">
						<a> {{ formatMessage(messages.learnMore) }} <RightArrowIcon /> </a>
					</ButtonStyled>
				</div>
			</template>
		</Admonition>
		<Admonition
			v-if="true"
			type="success"
			class="mb-4"
			:header="formatMessage(messages.completedTitle)"
			:body="formatMessage(messages.completedDescription)"
		/>
		<Admonition
			v-if="true"
			type="warning"
			class="mb-4"
			:header="formatMessage(messages.attentionNeededTitle)"
			:body="formatMessage(messages.attentionNeededDescriptionDraft)"
		/>
		<Admonition
			v-if="true"
			type="critical"
			class="mb-4"
			:header="formatMessage(messages.failTitle)"
			:body="formatMessage(messages.failDescription)"
		/>
		<div class="grid grid-cols-[1fr_auto] gap-2">
			<StyledInput
				v-model="searchQuery"
				type="search"
				:placeholder="
					formatMessage(messages.searchPlaceholder, {
						count: externalFiles.length,
					})
				"
				:icon="SearchIcon"
				input-class="h-[40px]"
			/>
			<div>
				<Combobox
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:options="sortTypes"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<SortAscIcon
								v-if="currentSortType === 'Oldest'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ currentSortType }}</span>
						</span>
					</template>
				</Combobox>
			</div>
		</div>
		<div class="mt-4 flex flex-col gap-3">
			<ExternalProjectPermissionsCard title="FTB Library" />
		</div>
	</template>
	<template v-else>
		<EmptyState
			:heading="formatMessage(messages.emptyStateHeading)"
			:description="formatMessage(messages.emptyStateDescription)"
			type="done"
		/>
	</template>
</template>
