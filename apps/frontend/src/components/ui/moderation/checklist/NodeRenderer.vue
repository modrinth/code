<script lang="ts" setup>
import type { Node, NodeContext, NodeState, NodeStateWithChildren } from '@modrinth/moderation'
import { ButtonStyled, Checkbox, MarkdownEditor, StyledInput } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { reactive, watchEffect } from 'vue'

const props = defineProps<{
	nodes: Node[]
	getState: (nodeId: string) => NodeState
	setState: (nodeId: string, value: NodeState) => void
	showContext: NodeContext
	onImageUpload?: (file: File) => Promise<string>
	flex?: boolean
}>()

function isVisible(node: Node): boolean {
	return !node.shown || node.shown(props.showContext)
}

function isEnabled(node: Node): boolean {
	return !node.enabled || node.enabled(props.showContext)
}

function getBooleanState(node: Node): boolean {
	const state = props.getState(node.id!)
	if (typeof state === 'boolean') return state
	if (state && typeof state === 'object' && !(state instanceof Set)) {
		const v = (state as NodeStateWithChildren).value
		if (typeof v === 'boolean') return v
	}
	return node.defaultChecked ?? false
}

function getMultiSelectState(node: Node): Set<string> {
	const state = props.getState(node.id!)
	return state instanceof Set ? state : new Set<string>()
}

function getSelectState(node: Node): string | undefined {
	const state = props.getState(node.id!)
	return typeof state === 'string' ? state : undefined
}

function toggleSelect(parent: Node, child: Node) {
	const current = getSelectState(parent)
	props.setState(parent.id!, current === child.id ? undefined : child.id)
}

function getTextState(node: Node): string {
	const state = props.getState(node.id!)
	return typeof state === 'string' ? state : ''
}

function toggleBoolean(node: Node) {
	const raw = props.getState(node.id!)
	const next = !getBooleanState(node)
	const isDefault = next === (node.defaultChecked ?? false)
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const { value: _v, ...children } = raw as NodeStateWithChildren & Record<string, NodeState>
		const hasChildren = Object.keys(children).length > 0
		props.setState(node.id!, isDefault && !hasChildren ? undefined : { ...children, ...(isDefault ? {} : { value: next }) } as NodeState)
	} else {
		props.setState(node.id!, isDefault ? undefined : next)
	}
}

function makeBooleanChildGetState(boolNode: Node): (childId: string) => NodeState {
	return (childId: string) => {
		const raw = props.getState(boolNode.id!)
		if (!raw || typeof raw !== 'object' || raw instanceof Set) return undefined
		return (raw as NodeStateWithChildren)[childId]
	}
}

function makeBooleanChildSetState(boolNode: Node): (childId: string, value: NodeState) => void {
	return (childId: string, value: NodeState) => {
		const raw = props.getState(boolNode.id!)
		const boolVal = getBooleanState(boolNode)
		const existing = (raw && typeof raw === 'object' && !(raw instanceof Set))
			? raw as NodeStateWithChildren
			: {} as NodeStateWithChildren
		if (value === undefined) {
			const { [childId]: _removed, value: _v, ...rest } = existing as Record<string, NodeState>
			const hasChildren = Object.keys(rest).length > 0
			props.setState(boolNode.id!, hasChildren
				? { ...rest, ...(boolVal ? { value: boolVal } : {}) } as NodeState
				: boolVal ? true : undefined)
		} else {
			props.setState(boolNode.id!, { ...existing, ...(boolVal ? { value: boolVal } : {}), [childId]: value } as NodeState)
		}
	}
}

function toggleChip(parent: Node, child: Node) {
	const selected = new Set(getMultiSelectState(parent))
	if (selected.has(child.id!)) {
		selected.delete(child.id!)
	} else {
		selected.add(child.id!)
	}
	props.setState(parent.id!, selected.size > 0 ? selected : undefined)
}

function getChildren(node: Node): Node[] {
	if (node.childrenFn) return node.childrenFn(props.showContext)
	return node.children ?? []
}

function visibleChildren(node: Node): Node[] {
	return getChildren(node).filter(isVisible)
}

function makeGroupGetState(groupNode: Node): (childId: string) => NodeState {
	return (childId: string) => {
		const raw = props.getState(groupNode.id!)
		if (!raw || typeof raw !== 'object' || raw instanceof Set) return undefined
		return (raw as NodeStateWithChildren)[childId]
	}
}

function makeGroupSetState(groupNode: Node): (childId: string, value: NodeState) => void {
	return (childId: string, value: NodeState) => {
		const raw = props.getState(groupNode.id!)
		const current = (raw && typeof raw === 'object' && !(raw instanceof Set))
			? raw as NodeStateWithChildren
			: undefined
		if (value === undefined) {
			if (!current) return
			const { [childId]: _removed, ...rest } = current as Record<string, NodeState>
			props.setState(groupNode.id!, Object.keys(rest).length > 0 ? rest as NodeState : undefined)
		} else {
			props.setState(groupNode.id!, { ...(current ?? {}), [childId]: value } as NodeState)
		}
	}
}

function renderProse(content: string): string {
	return renderString(content).replace(/<a /g, '<a target="_blank" ')
}

const proseContents = reactive(new Map<Node, string | null>())

watchEffect(async () => {
	for (const node of props.nodes) {
		if (node.type === 'prose' && node.content && isVisible(node)) {
			proseContents.set(node, await node.content(props.showContext))
		}
	}
})
</script>

<template>
	<div :class="flex ? 'flex flex-wrap gap-2' : 'space-y-4'">
		<template v-for="node in nodes" :key="node.id ?? node.type">
			<template v-if="isVisible(node)">
				<!-- group -->
				<NodeRenderer
					v-if="node.type === 'group'"
					:nodes="getChildren(node)"
					:get-state="node.id ? makeGroupGetState(node) : getState"
					:set-state="node.id ? makeGroupSetState(node) : setState"
					:show-context="showContext"
					:on-image-upload="onImageUpload"
					:flex="node.layout !== 'column'"
				/>

				<!-- button -->
				<ButtonStyled
					v-else-if="node.type === 'boolean' && node.variant === 'button'"
					:color="getBooleanState(node) ? 'brand' : 'standard'"
				>
					<button :disabled="!isEnabled(node)" @click="toggleBoolean(node)">{{ node.label }}</button>
				</ButtonStyled>

				<!-- toggle -->
				<Checkbox
					v-else-if="node.type === 'boolean' && node.variant === 'toggle'"
					:model-value="getBooleanState(node)"
					:label="node.label"
					:disabled="!isEnabled(node)"
					@update:model-value="isEnabled(node) && toggleBoolean(node)"
				/>

				<!-- select (single-choice) -->
				<div v-else-if="node.type === 'select'">
					<div class="mb-2 font-semibold">{{ node.label }}</div>
					<div class="flex flex-wrap gap-2">
						<template v-for="child in visibleChildren(node)" :key="child.id">
							<ButtonStyled :color="getSelectState(node) === child.id ? 'brand' : 'standard'">
								<button
									:disabled="!!getSelectState(node) && getSelectState(node) !== child.id"
									@click="toggleSelect(node, child)"
								>{{ child.label }}</button>
							</ButtonStyled>
						</template>
					</div>
					<template v-for="child in visibleChildren(node)" :key="`sub-${child.id}`">
						<NodeRenderer
							v-if="getSelectState(node) === child.id && child.children?.length"
							:nodes="child.children!"
							:get-state="getState"
							:set-state="setState"
							:show-context="showContext"
							:on-image-upload="onImageUpload"
							class="mt-2"
						/>
					</template>
				</div>

				<!-- multi-select (chips) -->
				<div v-else-if="node.type === 'multi-select'">
					<div class="mb-2 font-semibold">{{ node.label }}</div>
					<div class="flex flex-wrap gap-2">
						<template v-for="child in visibleChildren(node)" :key="child.id">
							<ButtonStyled
								:color="getMultiSelectState(node).has(child.id!) ? 'brand' : 'standard'"
								@click="toggleChip(node, child)"
							>
								<button>{{ child.label }}</button>
							</ButtonStyled>
						</template>
					</div>
					<template v-for="child in visibleChildren(node)" :key="`sub-${child.id}`">
						<NodeRenderer
							v-if="getMultiSelectState(node).has(child.id!) && child.children?.length"
							:nodes="child.children!"
							:get-state="getState"
							:set-state="setState"
							:show-context="showContext"
							:on-image-upload="onImageUpload"
							class="mt-2"
						/>
					</template>
				</div>

				<!-- text -->
				<div v-else-if="node.type === 'text'" class="inputs universal-labels">
					<label :for="`node-${node.id}`">
						<span class="label__title">
							{{ node.label }}<span v-if="node.required" class="required">*</span>
						</span>
					</label>
					<StyledInput
						:id="`node-${node.id}`"
						:model-value="getTextState(node)"
						:placeholder="node.placeholder"
						autocomplete="off"
						@update:model-value="(v: string) => setState(node.id!, v || undefined)"
					/>
				</div>

				<!-- markdown -->
				<div v-else-if="node.type === 'markdown'" class="inputs universal-labels">
					<label :for="`node-${node.id}`">
						<span class="label__title">
							{{ node.label }}<span v-if="node.required" class="required">*</span>
						</span>
					</label>
					<MarkdownEditor
						:id="`node-${node.id}`"
						:model-value="getTextState(node)"
						:placeholder="node.placeholder"
						:max-height="300"
						:disabled="false"
						:heading-buttons="false"
						:on-image-upload="onImageUpload"
						@update:model-value="(v: string) => setState(node.id!, v || undefined)"
					/>
				</div>

				<!-- prose (read-only markdown display) -->
				<div
					v-else-if="node.type === 'prose' && proseContents.get(node) != null"
					class="markdown-body"
					v-html="renderProse(proseContents.get(node)!)"
				/>

				<!-- label -->
				<p v-else-if="node.type === 'label'" class="text-secondary">{{ node.label }}</p>
			</template>
		</template>

		<!-- children of active boolean nodes, rendered after all siblings -->
		<template v-for="node in nodes" :key="`children-${node.id ?? node.type}`">
			<NodeRenderer
				v-if="isVisible(node) && node.type === 'boolean' && getBooleanState(node) && getChildren(node).length"
				:nodes="getChildren(node)"
				:get-state="makeBooleanChildGetState(node)"
				:set-state="makeBooleanChildSetState(node)"
				:show-context="showContext"
				:on-image-upload="onImageUpload"
				class="w-full"
			/>
		</template>
	</div>
</template>
