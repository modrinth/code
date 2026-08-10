import type {
	AnyNode,
	ChildNode,
	HasChildren,
	HasValue,
	Identified,
	NodePropsContext,
	NodeState,
	OnChangeFn,
	Reactive,
	TweakDef,
	Writer,
} from '@modrinth/moderation/src/types/node'
import {
	childWriter,
	getBooleanChildState,
	getEffectiveValue,
	hasCap,
	hasChildrenCap,
	hasIdCap,
	hasOptionsCap,
	hasValueCap,
	isNodeActive,
	isShown,
	originScope,
	resolve,
	resolveChildren,
	writeNodeValue,
	withStateDefaults,
} from '@modrinth/moderation/src/types/node'
import type { Component } from 'vue'
import { computed, watchEffect } from 'vue'

import { getDropdownMinWidth } from './dropdown-width'
import { resolveNodeRenderer } from './renderers'
import type { NodeRendererProps, RenderableValueNode, RendererPropsContext } from './types'

const TOOLTIP_BASE = {
	delay: { show: 500, hide: 0 },
	triggers: ['hover', 'focus'],
	placement: 'top',
}

export function useNodeRenderer(props: NodeRendererProps) {
	const wrappedState = computed(() => withStateDefaults(props.state, props.nodes, props.write))

	function resolveComponent(node: RenderableValueNode): Component | undefined {
		return resolveNodeRenderer(node)?.component
	}

	function titleClass(depth: number): string {
		if (depth === 0) return 'text-lg font-extrabold text-contrast'
		if (depth === 1) return 'text-base font-semibold'
		if (depth === 2) return 'text-sm font-semibold'
		return ''
	}

	function getTitle(node: object): string | undefined {
		if (!hasCap(node, '_title')) return undefined
		const title = node._title as Reactive<string> | undefined
		if (title === undefined) return undefined
		return resolve(title) || undefined
	}

	function needsAttention(node: object): boolean {
		return props.meta.attentionMap.get(node) ?? false
	}

	function isFixActionable(node: object): boolean {
		return props.meta.metaMap.get(node)?.isFixActionable ?? false
	}

	function isEnabled(node: object): boolean {
		if (!hasCap(node, '_enabled') || node._enabled === undefined) return true
		if (typeof node._enabled === 'function') {
			return (node._enabled as (state: Record<string, NodeState>) => boolean)(wrappedState.value)
		}
		return resolve(node._enabled as Reactive<boolean>)
	}

	function toggleSetValue(node: RenderableValueNode, value: string): void {
		const current = getEffectiveValue(
			node,
			props.state[node.id],
			wrappedState.value,
		) as unknown as string[]
		const set = new Set(Array.isArray(current) ? current : [])
		if (set.has(value)) set.delete(value)
		else set.add(value)
		writeNodeValue(node, props.state, props.write, Array.from(set) as never, wrappedState.value)
	}

	function resolveTooltip(node: object): Record<string, unknown> | undefined {
		if (hasCap(node, '_tooltip')) {
			const tooltip = node._tooltip as
				| Reactive<string>
				| ((state: Record<string, NodeState>) => string)
				| undefined
			if (tooltip !== undefined) {
				const content =
					typeof tooltip === 'function' ? tooltip(wrappedState.value) : resolve(tooltip)
				if (content) return { ...TOOLTIP_BASE, content }
			}
		}
		const html = hasCap(node, '_segments') ? props.meta.tooltipHtml.get(node) : undefined
		return html ? { ...TOOLTIP_BASE, content: html, html: true } : undefined
	}

	function componentProps(node: RenderableValueNode): Record<string, unknown> {
		const context: NodePropsContext = {
			onImageUpload: props.onImageUpload,
			toggleSetValue: (value) => toggleSetValue(node, value),
		}
		const rendererContext: RendererPropsContext = {
			...context,
			nodeFacts: {
				needsAttention: needsAttention(node),
				fixActionable: isFixActionable(node),
			},
		}
		const dropdownStyle = hasOptionsCap(node)
			? {
					class: '!w-auto max-w-full',
					style: {
						minWidth: getDropdownMinWidth(
							node._options as unknown as Array<{ label: string }>,
						),
					},
				}
			: undefined
		return {
			disabled: !isEnabled(node),
			...dropdownStyle,
			...resolveNodeRenderer(node)?.props?.(node, rendererContext),
			...node._extraProps?.(context),
		}
	}

	function containerScope(node: HasChildren & Partial<Identified>): {
		state: Record<string, NodeState>
		write: Writer
	} {
		if (hasCap(node, '_stateOrigin') && node._stateOrigin && props.globalState) {
			return originScope(props.globalState, node._stateOrigin as string[])
		}
		if (!hasIdCap(node)) return { state: props.state, write: props.write }
		const raw = props.state[node.id]
		const state =
			raw && typeof raw === 'object' && !(raw instanceof Set)
				? (raw as Record<string, NodeState>)
				: {}
		return { state, write: childWriter(props.state, props.write, node.id) }
	}

	function valueScope(node: HasValue & Identified): {
		state: Record<string, NodeState>
		write: Writer
	} {
		const state = getBooleanChildState(props.state[node.id])
		return { state, write: childWriter(props.state, props.write, node.id) }
	}

	function clickButton(node: object): void {
		if (!hasCap(node, '_onClick')) return
		;(node._onClick as (state: Record<string, NodeState>) => void)?.(wrappedState.value)
	}

	function buttonIcon(node: object): Component | undefined {
		return hasCap(node, '_icon') ? (node._icon as Component | undefined) : undefined
	}

	function buttonLabel(node: object): string {
		return hasCap(node, 'label') && typeof node.label === 'string' ? node.label : ''
	}

	function childLayout(node: object): 'flex' | 'column' | undefined {
		if (!hasCap(node, '_layout')) return undefined
		return node._layout === 'flex' || node._layout === 'column' ? node._layout : undefined
	}

	function tweakCurrent(node: RenderableValueNode): unknown {
		return getEffectiveValue(node, props.state[node.id], wrappedState.value)
	}

	function tweakResult(tweak: TweakDef, node: RenderableValueNode): unknown {
		return tweak.compute(tweakCurrent(node), wrappedState.value)
	}

	function tweakEnabled(tweak: TweakDef, node: RenderableValueNode): boolean {
		const result = tweakResult(tweak, node)
		return result !== null && result !== undefined && result !== tweakCurrent(node)
	}

	function tweakTooltip(
		tweak: TweakDef,
		node: RenderableValueNode,
	): Record<string, unknown> | undefined {
		if (!tweakEnabled(tweak, node)) return undefined
		const content = tweakResult(tweak, node)
		return content ? { ...TOOLTIP_BASE, content: String(content) } : undefined
	}

	function tweakLabel(tweak: TweakDef, node: RenderableValueNode): string {
		const result = tweakResult(tweak, node)
		return result !== null && result !== undefined ? String(result) : 'Apply suggested value'
	}

	function applyTweak(tweak: TweakDef, node: RenderableValueNode): void {
		const result = tweakResult(tweak, node)
		if (result !== null && result !== undefined) updateValue(node, result)
	}

	function nodeKey(item: ChildNode, index: number): string {
		return typeof item === 'object' && item !== null && hasIdCap(item)
			? item.id
			: `n-${index}`
	}

	function modelProp(item: object): string {
		return (item as RenderableValueNode)._modelProp
	}

	function updateEvent(item: object): string {
		return `update:${modelProp(item)}`
	}

	function updateValue(item: RenderableValueNode, value: unknown): void {
		const onChange = hasCap(item, '_onChange')
			? (item._onChange as OnChangeFn | undefined)
			: undefined
		if (onChange) {
			const result = onChange(value as string, { override: (override) => ({ __override: override }) })
			if (result && typeof result === 'object' && '__override' in result) {
				writeNodeValue(
					item,
					props.state,
					props.write,
					result.__override as never,
					wrappedState.value,
				)
				return
			}
		}
		writeNodeValue(item, props.state, props.write, value as never, wrappedState.value)
	}

	const seenOnChangeValues = new Map<object, unknown>()
	watchEffect(() => {
		for (const node of props.nodes) {
			if (typeof node !== 'object' || node === null) continue
			if (!hasCap(node, '_onChange') || !node._onChange) continue
			if (!hasValueCap(node) || !hasIdCap(node) || !isShown(node as AnyNode)) continue
			const value = getEffectiveValue(node, props.state[node.id], wrappedState.value)
			if (seenOnChangeValues.has(node) && seenOnChangeValues.get(node) === value) continue
			seenOnChangeValues.set(node, value)
			const onChange = (node as RenderableValueNode)._onChange as OnChangeFn | undefined
			onChange?.(value as never, { override: (override) => ({ __override: override }) })
		}
	})

	return {
		applyTweak,
		buttonIcon,
		buttonLabel,
		clickButton,
		childLayout,
		componentProps,
		containerScope,
		getEffectiveValue,
		getTitle,
		hasCap,
		hasChildrenCap,
		hasIdCap,
		hasValueCap,
		isEnabled,
		isNodeActive,
		isShown,
		modelProp,
		needsAttention,
		nodeKey,
		resolveChildren,
		resolveComponent,
		resolveTooltip,
		titleClass,
		tweakEnabled,
		tweakLabel,
		tweakTooltip,
		updateEvent,
		updateValue,
		valueScope,
		wrappedState,
	}
}
