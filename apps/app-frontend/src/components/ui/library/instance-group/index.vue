<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { Accordion, TagItem } from '@modrinth/ui'
import { ref } from 'vue'

import Instance from '@/components/ui/library/instance-group/instance.vue'
import { useLibrary } from '@/components/ui/library/use-library'
import type {
	InstanceCard,
	InstanceGroup as InstanceGroupType,
} from '@/components/ui/library/use-library'

defineProps<{
	instanceGroup: InstanceGroupType
}>()

const { isSectionCollapsed, setSectionCollapsed, handleInstanceContextMenu } = useLibrary()

const instanceComponents = ref<InstanceCard[]>([])

function openInstanceContextMenu(event: MouseEvent, instanceId: string, instanceGroupName: string) {
	const instanceComponent = instanceComponents.value.find(
		(component) => component.instance.id === instanceId,
	)
	if (!instanceComponent) return

	handleInstanceContextMenu(event, instanceComponent, instanceGroupName)
}
</script>

<template>
	<Accordion
		:open-by-default="!isSectionCollapsed(instanceGroup.key)"
		button-class="group flex w-full cursor-pointer items-center gap-2 border-0 border-b border-solid border-b-surface-5 bg-transparent px-0 py-2.5 mb-3 text-left"
		class="w-full"
		@on-open="setSectionCollapsed(instanceGroup.key, false)"
		@on-close="setSectionCollapsed(instanceGroup.key, true)"
	>
		<template v-if="instanceGroup.key !== 'None'" #button="{ open }">
			<DropdownIcon
				class="size-5 shrink-0 text-secondary duration-300 group-hover:text-primary transition-all"
				:class="{ 'rotate-180': open }"
			/>
			<span class="text-base font-semibold text-primary group-hover:text-contrast transition-all">
				{{ instanceGroup.key }}
			</span>
			<TagItem class="shrink-0 bg-surface-2 border-surface-3">
				{{ instanceGroup.instances.length }}
			</TagItem>
		</template>
		<p
			v-if="instanceGroup.instances.length === 0"
			class="m-0 py-2.5 pl-0.5 text-base font-medium text-secondary"
		>
			No instances in this group.
		</p>
		<section
			v-else
			class="grid w-full grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-3 overflow-y-auto scroll-smooth"
		>
			<Instance
				v-for="instance in instanceGroup.instances"
				ref="instanceComponents"
				:key="instance.id + instance.install_stage"
				:instance="instance"
				@contextmenu.prevent.stop="
					(event: MouseEvent) => openInstanceContextMenu(event, instance.id, instanceGroup.key)
				"
			/>
		</section>
	</Accordion>
</template>
