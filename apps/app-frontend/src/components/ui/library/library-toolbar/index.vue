<script setup lang="ts">
import { PlusIcon, SearchIcon } from '@modrinth/assets'
import { ButtonStyled, StyledInput } from '@modrinth/ui'

import FilterMenu from '@/components/ui/library/library-toolbar/filter-menu.vue'
import NewGroupModal from '@/components/ui/library/library-toolbar/new-group-modal.vue'
import SortMenu from '@/components/ui/library/library-toolbar/sort-menu.vue'
import { useLibrary } from '@/components/ui/library/use-library'

const { search, openNewGroupModal } = useLibrary()
</script>

<template>
	<div class="flex flex-wrap gap-2">
		<StyledInput
			v-model="search"
			:icon="SearchIcon"
			type="text"
			placeholder="Search"
			clearable
			wrapper-class="min-w-[16rem] flex-1"
		/>
		<FilterMenu />
		<SortMenu />
		<ButtonStyled>
			<button @click="openNewGroupModal">
				<PlusIcon />
				Create group
			</button>
		</ButtonStyled>
	</div>
	<NewGroupModal />
</template>
