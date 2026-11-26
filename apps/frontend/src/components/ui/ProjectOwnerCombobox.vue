<template>
	<Combobox
		v-model="owner"
		class="project-owner-combobox"
		:options="options"
		:disabled="disabled"
		:show-chevron="!hideChevron"
		searchable
		select-search-text-on-focus
		search-type="search"
		search-inputmode="search"
		search-autocomplete="off"
		search-autocorrect="off"
		search-autocapitalize="none"
		:search-spellcheck="false"
		:search-input-id="inputId"
		:search-input-attrs="{
			'data-1p-ignore': 'true',
			'data-bwignore': 'true',
			'data-form-type': 'other',
			'data-lpignore': 'true',
			'data-protonpass-ignore': 'true',
		}"
	>
		<template #option="{ item }">
			<div class="flex min-w-0 items-center gap-2">
				<Avatar
					:src="optionsByValue.get(item.value)?.avatar_url"
					:circle="!optionsByValue.get(item.value)?.isOrganization"
					size="1.5rem"
					alt=""
					no-shadow
				/>
				<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
				<OrganizationIcon
					v-if="optionsByValue.get(item.value)?.isOrganization"
					role="img"
					:aria-label="formatMessage(organizationLabel)"
					class="size-4 shrink-0"
				/>
			</div>
		</template>
	</Combobox>
</template>

<script setup lang="ts">
import { OrganizationIcon } from '@modrinth/assets'
import { Avatar, Combobox, defineMessage, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps<{
	inputId: string
	options: { value: string; label: string; avatar_url?: string | null; isOrganization?: boolean }[]
	disabled?: boolean
	hideChevron?: boolean
}>()
const owner = defineModel<string>()
const optionsByValue = computed(
	() => new Map(props.options.map((option) => [option.value, option])),
)
const { formatMessage } = useVIntl()
const organizationLabel = defineMessage({
	id: 'organization.label.organization',
	defaultMessage: 'Organization',
})
</script>

<style scoped>
.project-owner-combobox :deep(input[type='search']::-webkit-search-cancel-button) {
	display: none;
}
</style>
