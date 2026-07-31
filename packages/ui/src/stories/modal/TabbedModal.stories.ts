import {
	CoffeeIcon,
	GameIcon,
	GaugeIcon,
	InfoIcon,
	LanguagesIcon,
	MonitorIcon,
	PaintbrushIcon,
	ReportIcon,
	SettingsIcon,
	ShieldIcon,
	WrenchIcon,
} from '@modrinth/assets'
import type { StoryObj } from '@storybook/vue3-vite'
import { defineComponent, h, ref } from 'vue'

import { Button } from '../../components/base/buttons'
import UnsavedChangesPopup from '../../components/base/UnsavedChangesPopup.vue'
import TabbedModal from '../../components/modal/TabbedModal.vue'

function makeTabContent(label: string, lines = 3) {
	return defineComponent({
		name: `${label}Tab`,
		render() {
			return h('div', { class: 'space-y-4 py-2' }, [
				h('h2', { class: 'text-xl font-bold text-contrast m-0' }, label),
				...Array.from({ length: lines }, (_, i) =>
					h('p', { class: 'text-secondary m-0' }, `${label} content paragraph ${i + 1}.`),
				),
			])
		},
	})
}

const meta = {
	title: 'Modal/TabbedModal',
	// @ts-ignore
	component: TabbedModal,
}

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { TabbedModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					icon: InfoIcon,
					content: makeTabContent('General'),
				},
				{
					name: { id: 'appearance', defaultMessage: 'Appearance' },
					icon: PaintbrushIcon,
					content: makeTabContent('Appearance'),
				},
				{
					name: { id: 'privacy', defaultMessage: 'Privacy' },
					icon: ShieldIcon,
					content: makeTabContent('Privacy'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open Tabbed Modal</Button>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const WithTitleSlot: StoryObj = {
	render: () => ({
		components: { TabbedModal, SettingsIcon, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					icon: InfoIcon,
					content: makeTabContent('General'),
				},
				{
					name: { id: 'appearance', defaultMessage: 'Appearance' },
					icon: PaintbrushIcon,
					content: makeTabContent('Appearance'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open with Title Slot</Button>
				<TabbedModal ref="modalRef" :tabs="tabs">
					<template #title>
						<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
							<SettingsIcon /> Custom Title
						</span>
					</template>
				</TabbedModal>
			</div>
		`,
	}),
}

export const WithFooter: StoryObj = {
	render: () => ({
		components: { TabbedModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					icon: InfoIcon,
					content: makeTabContent('General'),
				},
				{
					name: { id: 'appearance', defaultMessage: 'Appearance' },
					icon: PaintbrushIcon,
					content: makeTabContent('Appearance'),
				},
				{
					name: { id: 'privacy', defaultMessage: 'Privacy' },
					icon: ShieldIcon,
					content: makeTabContent('Privacy'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open with Footer</Button>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs">
					<template #footer>
						<div class="mt-auto text-secondary text-sm">
							<p class="m-0">App v1.0.0</p>
							<p class="m-0">macOS 15.0</p>
						</div>
					</template>
				</TabbedModal>
			</div>
		`,
	}),
}

export const WithFloatingActionBar: StoryObj = {
	render: () => ({
		components: { TabbedModal, UnsavedChangesPopup, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const dirty = ref(true)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					icon: InfoIcon,
					content: makeTabContent('General', 20),
				},
			]
			return { modalRef, dirty, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="dirty = true; modalRef?.show()">Open with Floating Action Bar</Button>
				<TabbedModal
					ref="modalRef"
					header="Settings"
					:tabs="tabs"
					:floating-action-bar-shown="dirty"
				>
					<template #floating-action-bar>
						<UnsavedChangesPopup
							:original="{ dirty: false }"
							:modified="{ dirty }"
							inline
							@save="dirty = false"
							@reset="dirty = false"
						/>
					</template>
				</TabbedModal>
			</div>
		`,
	}),
}

export const WithBadge: StoryObj = {
	render: () => ({
		components: { TabbedModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					icon: InfoIcon,
					content: makeTabContent('General'),
				},
				{
					name: { id: 'language', defaultMessage: 'Language' },
					icon: LanguagesIcon,
					content: makeTabContent('Language'),
					badge: { id: 'beta', defaultMessage: 'Beta' },
				},
				{
					name: { id: 'privacy', defaultMessage: 'Privacy' },
					icon: ShieldIcon,
					content: makeTabContent('Privacy'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open with Badge</Button>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const HiddenTabs: StoryObj = {
	render: () => ({
		components: { TabbedModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					icon: InfoIcon,
					content: makeTabContent('General'),
				},
				{
					name: { id: 'hidden', defaultMessage: 'Hidden Tab' },
					icon: ReportIcon,
					content: makeTabContent('Hidden'),
					shown: false,
				},
				{
					name: { id: 'appearance', defaultMessage: 'Appearance' },
					icon: PaintbrushIcon,
					content: makeTabContent('Appearance'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open with Hidden Tab</Button>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const ManyTabs: StoryObj = {
	render: () => ({
		components: { TabbedModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'general', defaultMessage: 'General' },
					category: { id: 'display-category', defaultMessage: 'Display' },
					icon: InfoIcon,
					content: makeTabContent('General'),
				},
				{
					name: { id: 'appearance', defaultMessage: 'Appearance' },
					category: { id: 'display-category', defaultMessage: 'Display' },
					icon: PaintbrushIcon,
					content: makeTabContent('Appearance'),
				},
				{
					name: { id: 'language', defaultMessage: 'Language' },
					category: { id: 'display-category', defaultMessage: 'Display' },
					icon: LanguagesIcon,
					content: makeTabContent('Language'),
				},
				{
					name: { id: 'privacy', defaultMessage: 'Privacy' },
					category: { id: 'account-category', defaultMessage: 'Account' },
					icon: ShieldIcon,
					content: makeTabContent('Privacy'),
				},
				{
					name: { id: 'java', defaultMessage: 'Java and memory' },
					category: { id: 'instances-category', defaultMessage: 'Instances' },
					icon: CoffeeIcon,
					content: makeTabContent('Java and memory'),
				},
				{
					name: { id: 'instances', defaultMessage: 'Default instance options' },
					category: { id: 'instances-category', defaultMessage: 'Instances' },
					icon: GameIcon,
					content: makeTabContent('Default instance options'),
				},
				{
					name: { id: 'resources', defaultMessage: 'Resource management' },
					category: { id: 'instances-category', defaultMessage: 'Instances' },
					icon: GaugeIcon,
					content: makeTabContent('Resource management'),
				},
				{
					name: { id: 'window', defaultMessage: 'Window' },
					category: { id: 'advanced-category', defaultMessage: 'Advanced' },
					icon: MonitorIcon,
					content: makeTabContent('Window'),
				},
				{
					name: { id: 'hooks', defaultMessage: 'Launch hooks' },
					category: { id: 'advanced-category', defaultMessage: 'Advanced' },
					icon: WrenchIcon,
					content: makeTabContent('Launch hooks'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open with Many Tabs</Button>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const ScrollableContent: StoryObj = {
	render: () => ({
		components: { TabbedModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)
			const tabs = [
				{
					name: { id: 'long', defaultMessage: 'Long content' },
					icon: InfoIcon,
					content: makeTabContent('Long content', 30),
				},
				{
					name: { id: 'short', defaultMessage: 'Short content' },
					icon: PaintbrushIcon,
					content: makeTabContent('Short content', 2),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="brand" @click="modalRef?.show()">Open with Scrollable Content</Button>
				<TabbedModal ref="modalRef" header="Scrollable Demo" :tabs="tabs" />
			</div>
		`,
	}),
}
