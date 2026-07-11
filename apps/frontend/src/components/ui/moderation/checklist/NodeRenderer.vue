<script lang="ts" setup>
import type {
	BooleanNodeBuilder,
	ContentFn,
	DisplayNodeBuilder,
	GroupNodeBuilder,
	IdentifiedNodeBuilder,
	InputNodeBuilder,
	NodeBuilder,
	NodeContext,
	NodeState,
	NodeStateWithChildren,
	ValueNodeBuilder,
} from '@modrinth/moderation'
import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
} from '@modrinth/moderation'
import { ButtonStyled, Checkbox, MarkdownEditor, StyledInput } from '@modrinth/ui'
import { renderHighlightedString, renderString } from '@modrinth/utils'
import { reactive, watchEffect } from 'vue'

const props = defineProps<{
	nodes: NodeBuilder[]
	getState: (nodeId: string) => NodeState
	setState: (nodeId: string, value: NodeState) => void
	showContext: NodeContext
	onImageUpload?: (file: File) => Promise<string>
	flex?: boolean
}>()

function isVisible(node: NodeBuilder): boolean {
	return !node._shown || node._shown(props.showContext)
}

function isEnabled(node: IdentifiedNodeBuilder): boolean {
	return !node._enabled || node._enabled(props.showContext)
}

function asBool(node: NodeBuilder): BooleanNodeBuilder {
	return node as BooleanNodeBuilder
}

function asIdentified(node: NodeBuilder): IdentifiedNodeBuilder {
	return node as IdentifiedNodeBuilder
}

function asGroup(node: NodeBuilder): GroupNodeBuilder {
	return node as GroupNodeBuilder
}

function asDisplay(node: NodeBuilder): DisplayNodeBuilder {
	return node as DisplayNodeBuilder
}

function asValue(node: NodeBuilder): ValueNodeBuilder {
	return node as ValueNodeBuilder
}

function asInput(node: NodeBuilder): InputNodeBuilder {
	return node as InputNodeBuilder
}

function getBooleanState(node: BooleanNodeBuilder): boolean {
	const state = props.getState(node.id!)
	if (typeof state === 'boolean') return state
	if (state && typeof state === 'object' && !(state instanceof Set)) {
		const v = (state as NodeStateWithChildren).value
		if (typeof v === 'boolean') return v
	}
	return (node._defaultValue as boolean | undefined) ?? false
}

function getMultiSelectState(node: IdentifiedNodeBuilder): Set<string> {
	const state = props.getState(node.id!)
	return state instanceof Set ? state : new Set<string>()
}

function getSelectState(node: IdentifiedNodeBuilder): string | undefined {
	const state = props.getState(node.id!)
	return typeof state === 'string' ? state : undefined
}

function toggleSelect(parent: IdentifiedNodeBuilder, child: IdentifiedNodeBuilder) {
	const current = getSelectState(parent)
	props.setState(parent.id!, current === child.id ? undefined : child.id)
}

function getTextState(node: IdentifiedNodeBuilder): string {
	const state = props.getState(node.id!)
	return typeof state === 'string' ? state : ''
}

function toggleBoolean(node: BooleanNodeBuilder) {
	const raw = props.getState(node.id!)
	const next = !getBooleanState(node)
	const defaultVal = (node._defaultValue as boolean | undefined) ?? false
	const isDefault = next === defaultVal
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const { value: _v, ...children } = raw as NodeStateWithChildren & Record<string, NodeState>
		const hasChildren = Object.keys(children).length > 0
		props.setState(node.id!, isDefault && !hasChildren ? undefined : { ...children, ...(isDefault ? {} : { value: next }) } as NodeState)
	} else {
		props.setState(node.id!, isDefault ? undefined : next)
	}
}

function makeBooleanChildGetState(boolNode: BooleanNodeBuilder): (childId: string) => NodeState {
	return (childId: string) => {
		const raw = props.getState(boolNode.id!)
		if (!raw || typeof raw !== 'object' || raw instanceof Set) return undefined
		return (raw as NodeStateWithChildren)[childId]
	}
}

function makeBooleanChildSetState(boolNode: BooleanNodeBuilder): (childId: string, value: NodeState) => void {
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

function toggleChip(parent: IdentifiedNodeBuilder, child: IdentifiedNodeBuilder) {
	const selected = new Set(getMultiSelectState(parent))
	if (selected.has(child.id!)) {
		selected.delete(child.id!)
	} else {
		selected.add(child.id!)
	}
	props.setState(parent.id!, selected.size > 0 ? selected : undefined)
}

function getChildren(node: IdentifiedNodeBuilder): NodeBuilder[] {
	if (node._childrenFn) return node._childrenFn(props.showContext)
	return node._children
}

function visibleChildren(node: IdentifiedNodeBuilder): NodeBuilder[] {
	return getChildren(node).filter(isVisible)
}

function makeGroupGetState(groupNode: IdentifiedNodeBuilder): (childId: string) => NodeState {
	return (childId: string) => {
		const raw = props.getState(groupNode.id!)
		if (!raw || typeof raw !== 'object' || raw instanceof Set) return undefined
		return (raw as NodeStateWithChildren)[childId]
	}
}

function makeGroupSetState(groupNode: IdentifiedNodeBuilder): (childId: string, value: NodeState) => void {
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

const tooltipHtml = reactive(new Map<NodeBuilder, string>())

function getNodeTooltipConfig(node: BooleanNodeBuilder) {
	const html = tooltipHtml.get(node)
	if (!html) return undefined
	return {
		content: html,
		html: true,
		delay: { show: 500, hide: 0 },
		triggers: ['hover', 'focus'],
		placement: 'top',
	}
}

const proseContents = reactive(new Map<NodeBuilder, string | null>())

watchEffect(async () => {
	// Read all reactive state synchronously before any await so Vue tracks dependencies
	const asyncDisplayTasks: Array<{ node: DisplayNodeBuilder; ctx: NodeContext }> = []
	const buttonTasks: Array<{ node: BooleanNodeBuilder; ctx: NodeContext }> = []

	for (const node of props.nodes) {
		if (node.type === 'display' && isVisible(node)) {
			const display = asDisplay(node)
			if (typeof display._content === 'string') {
				proseContents.set(node, display._content)
			} else {
				asyncDisplayTasks.push({ node: display, ctx: props.showContext })
			}
		}
		if (node.type === 'button' && isVisible(node)) {
			const boolNode = asBool(node)
			if (boolNode._action?._message) {
				const nodeState = props.getState(boolNode.id!)
				const childState = (nodeState && typeof nodeState === 'object' && !(nodeState instanceof Set))
					? (() => { const { value: _v, ...rest } = nodeState as NodeStateWithChildren & Record<string, NodeState>; return rest })()
					: {}
				buttonTasks.push({ node: boolNode, ctx: { ...props.showContext, state: childState } })
			}
		}
	}

	for (const { node, ctx } of asyncDisplayTasks) {
		proseContents.set(node, await (node._content as ContentFn)(ctx))
	}

	for (const { node, ctx } of buttonTasks) {
		try {
			const raw = await node._action!._message!(ctx)
			const expanded = expandVariables(raw, props.showContext.projectV2, props.showContext.project, {
				...flattenStaticVariables(),
				...flattenProjectVariables(props.showContext.projectV2),
				...flattenProjectV3Variables(props.showContext.project),
			})
			const trimmed = expanded.trim()
			tooltipHtml.set(node, trimmed
				? `<div class="markdown-body moderation-tooltip-markdown">${renderHighlightedString(trimmed)}</div>`
				: '',
			)
		} catch {
			tooltipHtml.set(node, '')
		}
	}
})
</script>

<template>
	<div :class="flex ? 'flex flex-wrap gap-2' : 'space-y-4'">
		<template v-for="node in nodes" :key="asIdentified(node).id ?? node.type">
			<template v-if="isVisible(node)">
				<!-- group -->
				<NodeRenderer
					v-if="node.type === 'group'"
					:nodes="getChildren(asIdentified(node))"
					:get-state="asGroup(node).id ? makeGroupGetState(asIdentified(node)) : getState"
					:set-state="asGroup(node).id ? makeGroupSetState(asIdentified(node)) : setState"
					:show-context="showContext"
					:on-image-upload="onImageUpload"
					:flex="asGroup(node)._layout !== 'column'"
				/>

				<!-- button -->
				<ButtonStyled
					v-else-if="node.type === 'button'"
					:color="getBooleanState(asBool(node)) ? 'brand' : 'standard'"
				>
					<button
						v-tooltip="getNodeTooltipConfig(asBool(node))"
						:disabled="!isEnabled(asIdentified(node))"
						@click="toggleBoolean(asBool(node))"
					>{{ asIdentified(node).label }}</button>
				</ButtonStyled>

				<!-- toggle -->
				<Checkbox
					v-else-if="node.type === 'toggle'"
					:model-value="getBooleanState(asBool(node))"
					:label="asIdentified(node).label"
					:disabled="!isEnabled(asIdentified(node))"
					@update:model-value="isEnabled(asIdentified(node)) && toggleBoolean(asBool(node))"
				/>

				<!-- select (single-choice) -->
				<div v-else-if="node.type === 'select'">
					<div class="mb-2 font-semibold">{{ asIdentified(node).label }}</div>
					<div class="flex flex-wrap gap-2">
						<template v-for="child in visibleChildren(asIdentified(node))" :key="asIdentified(child).id">
							<ButtonStyled :color="getSelectState(asIdentified(node)) === asIdentified(child).id ? 'brand' : 'standard'">
								<button
									:disabled="!!getSelectState(asIdentified(node)) && getSelectState(asIdentified(node)) !== asIdentified(child).id"
									@click="toggleSelect(asIdentified(node), asIdentified(child))"
								>{{ asIdentified(child).label }}</button>
							</ButtonStyled>
						</template>
					</div>
					<template v-for="child in visibleChildren(asIdentified(node))" :key="`sub-${asIdentified(child).id}`">
						<NodeRenderer
							v-if="getSelectState(asIdentified(node)) === asIdentified(child).id && getChildren(asIdentified(child)).length"
							:nodes="getChildren(asIdentified(child))"
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
					<div class="mb-2 font-semibold">{{ asIdentified(node).label }}</div>
					<div class="flex flex-wrap gap-2">
						<template v-for="child in visibleChildren(asIdentified(node))" :key="asIdentified(child).id">
							<ButtonStyled
								:color="getMultiSelectState(asIdentified(node)).has(asIdentified(child).id!) ? 'brand' : 'standard'"
								@click="toggleChip(asIdentified(node), asIdentified(child))"
							>
								<button>{{ asIdentified(child).label }}</button>
							</ButtonStyled>
						</template>
					</div>
					<template v-for="child in visibleChildren(asIdentified(node))" :key="`sub-${asIdentified(child).id}`">
						<NodeRenderer
							v-if="getMultiSelectState(asIdentified(node)).has(asIdentified(child).id!) && getChildren(asIdentified(child)).length"
							:nodes="getChildren(asIdentified(child))"
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
					<label :for="`node-${asIdentified(node).id}`">
						<span class="label__title">
							{{ asIdentified(node).label }}<span v-if="asValue(node)._required" class="required">*</span>
						</span>
					</label>
					<StyledInput
						:id="`node-${asIdentified(node).id}`"
						:model-value="getTextState(asIdentified(node))"
						:placeholder="asInput(node)._placeholder"
						autocomplete="off"
						@update:model-value="(v: string) => setState(asIdentified(node).id!, v || undefined)"
					/>
				</div>

				<!-- markdown -->
				<div v-else-if="node.type === 'markdown'" class="inputs universal-labels">
					<label :for="`node-${asIdentified(node).id}`">
						<span class="label__title">
							{{ asIdentified(node).label }}<span v-if="asValue(node)._required" class="required">*</span>
						</span>
					</label>
					<MarkdownEditor
						:id="`node-${asIdentified(node).id}`"
						:model-value="getTextState(asIdentified(node))"
						:placeholder="asInput(node)._placeholder"
						:max-height="300"
						:disabled="false"
						:heading-buttons="false"
						:on-image-upload="onImageUpload"
						@update:model-value="(v: string) => setState(asIdentified(node).id!, v || undefined)"
					/>
				</div>

				<!-- display (prose + label) -->
				<div
					v-else-if="node.type === 'display' && proseContents.get(node) != null"
					class="markdown-body"
					v-html="renderProse(proseContents.get(node)!)"
				/>
			</template>
		</template>

		<!-- children of active boolean nodes, rendered after all siblings -->
		<template v-for="node in nodes" :key="`children-${asIdentified(node).id ?? node.type}`">
			<NodeRenderer
				v-if="isVisible(node) && (node.type === 'button' || node.type === 'toggle') && getBooleanState(asBool(node)) && getChildren(asIdentified(node)).length"
				:nodes="getChildren(asIdentified(node))"
				:get-state="makeBooleanChildGetState(asBool(node))"
				:set-state="makeBooleanChildSetState(asBool(node))"
				:show-context="showContext"
				:on-image-upload="onImageUpload"
				class="w-full"
			/>
		</template>
	</div>
</template>
