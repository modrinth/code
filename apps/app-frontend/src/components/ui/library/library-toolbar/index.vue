<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { StyledInput } from '@modrinth/ui'

import FilterMenu from '@/components/ui/library/library-toolbar/filter-menu.vue'
import NewGroupModal from '@/components/ui/library/library-toolbar/new-group-modal.vue'
import SortMenu from '@/components/ui/library/library-toolbar/sort-menu.vue'
import { useLibrary } from '@/components/ui/library/use-library'

const { search, isNewGroupModalOpen, closeNewGroupModal } = useLibrary()
</script>

<template>
	<div class="flex gap-2">
		<StyledInput
			v-model="search"
			:icon="SearchIcon"
			type="text"
			placeholder="Search"
			clearable
			wrapper-class="flex-1"
		/>
		<FilterMenu />
		<SortMenu />
	</div>
	<NewGroupModal :open="isNewGroupModalOpen" @close="closeNewGroupModal" />
</template>
