import type { BuiltinRendererKey } from '@modrinth/moderation/src/types/node'
import { Checkbox, Combobox, MarkdownEditor, StyledInput, Toggle } from '@modrinth/ui'
import type { Component } from 'vue'

import LoaderPicker from '~/components/ui/create-project-version/components/LoaderPicker.vue'
import McVersionPicker from '~/components/ui/create-project-version/components/McVersionPicker.vue'

import ActionButton from '../action-button.vue'
import type { RenderableValueNode, RendererPropsContext } from './types'

interface RendererDefinition {
	component: Component
	props?: (node: RenderableValueNode, context: RendererPropsContext) => Record<string, unknown>
}

const builtinRenderers = {
	action: {
		component: ActionButton,
		props: (node, context) => ({
			label: 'label' in node && typeof node.label === 'string' ? node.label : '',
			icon: '_icon' in node ? node._icon : undefined,
			needsAttention: context.nodeFacts.needsAttention,
			fixActionable: context.nodeFacts.fixActionable,
		}),
	},
	checkbox: {
		component: Checkbox,
		props: (node) => ({
			label: 'label' in node && typeof node.label === 'string' ? node.label : '',
		}),
	},
	toggle: { component: Toggle },
	dropdown: {
		component: Combobox,
		props: (node) => {
			if (!('_options' in node) || !Array.isArray(node._options)) return {}
			const options = node._options as Array<{ value: string; label: string }>
			const none = '_none' in node && typeof node._none === 'string' ? node._none : undefined
			return {
				options: [
					...(none !== undefined ? [{ value: '', label: none }] : []),
					...options.map((option) => ({
						value: option.value,
						label: option.label,
					})),
				],
				triggerClass:
					'!bg-[var(--color-button-bg)] !rounded-[var(--radius-md)] !shadow-[var(--shadow-inset-sm),0_0_0_0_transparent]',
				dropdownClass: '!rounded-[var(--radius-md)] !bg-[var(--color-button-bg)] !border-0',
			}
		},
	},
	text: {
		component: StyledInput,
		props: () => ({ class: 'min-w-40 flex-1', autocomplete: 'off' }),
	},
	markdown: {
		component: MarkdownEditor,
		props: (_node, context) => ({
			maxHeight: 300,
			disabled: false,
			headingButtons: false,
			onImageUpload: context.onImageUpload,
		}),
	},
} satisfies Record<BuiltinRendererKey, RendererDefinition>

const customRenderers = {
	'loader-picker': LoaderPicker,
	'game-version-picker': McVersionPicker,
} satisfies Record<string, Component>

export function resolveNodeRenderer(node: RenderableValueNode): RendererDefinition | undefined {
	if (node._renderer.type === 'custom') {
		const component = customRenderers[node._renderer.key as keyof typeof customRenderers]
		return component ? { component } : undefined
	}
	return builtinRenderers[node._renderer.type]
}
