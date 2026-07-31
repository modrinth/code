<template>
	<ButtonStyled size="large" circular>
		<PopoutMenu
			v-if="authUser"
			:tooltip="
				saved ? formatMessage(commonMessages.savedLabel) : formatMessage(commonMessages.saveButton)
			"
			from="top-right"
			:aria-label="formatMessage(commonMessages.saveButton)"
			:dropdown-id="`${baseId}-save`"
		>
			<BookmarkIcon aria-hidden="true" :fill="saved ? 'currentColor' : 'none'" />
			<template #menu>
				<StyledInput
					v-model="displayCollectionsSearch"
					:placeholder="formatMessage(commonMessages.searchPlaceholder)"
					wrapper-class="menu-search"
				/>
				<div v-if="filteredCollections.length > 0" class="collections-list text-primary">
					<Checkbox
						v-for="option in filteredCollections"
						:key="option.id"
						:model-value="option.projects.includes(projectId)"
						class="popout-checkbox"
						@update:model-value="() => collectProject(option, projectId)"
					>
						{{ option.name }}
					</Checkbox>
				</div>

				<div v-else class="menu-text">
					<p class="popout-text">{{ noCollectionsLabel }}</p>
				</div>
				<ButtonStyled>
					<button class="mx-3 mb-3" @click="createCollection">
						<PlusIcon aria-hidden="true" />
						{{ createNewCollectionLabel }}
					</button>
				</ButtonStyled>
			</template>
		</PopoutMenu>
		<nuxt-link
			v-else
			v-tooltip="formatMessage(commonMessages.saveButton)"
			:to="signInRoute"
			:aria-label="formatMessage(commonMessages.saveButton)"
		>
			<BookmarkIcon aria-hidden="true" />
		</nuxt-link>
	</ButtonStyled>
</template>

<script setup lang="ts">
import { BookmarkIcon, PlusIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	commonMessages,
	PopoutMenu,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

type CollectionOption = {
	id: string
	name: string
	projects: string[]
}

const props = defineProps<{
	authUser?: unknown
	signInRoute: RouteLocationRaw
	projectId: string
	collections: CollectionOption[]
	saved: boolean
	baseId: string
	noCollectionsLabel: string
	createNewCollectionLabel: string
	collectProject: (option: CollectionOption, projectId: string) => void | Promise<void>
	createCollection: (event: MouseEvent) => void
}>()

const { formatMessage } = useVIntl()
const displayCollectionsSearch = ref('')

const filteredCollections = computed(() =>
	props.collections
		.filter((collection) =>
			collection.name.toLowerCase().includes(displayCollectionsSearch.value.toLowerCase()),
		)
		.slice()
		.sort((a, b) => a.name.localeCompare(b.name)),
)
</script>

<style scoped lang="scss">
.popout-checkbox {
	padding: var(--gap-sm) var(--gap-md);
	white-space: nowrap;

	&:hover {
		filter: brightness(0.95);
	}
}

.menu-text {
	padding: 0 var(--gap-md);
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.menu-search {
	margin: var(--gap-sm) var(--gap-md);
	width: calc(100% - var(--gap-md) * 2);
}

.collections-list {
	max-height: 40rem;
	overflow-y: auto;
	background-color: var(--color-bg);
	border-radius: var(--radius-md);
	margin: var(--gap-sm) var(--gap-md);
	padding: var(--gap-sm);
}
</style>
