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

import ButtonStyled from '../../components/base/ButtonStyled.vue'
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
		components: { TabbedModal, ButtonStyled },
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
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open Tabbed Modal</button>
				</ButtonStyled>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const WithTitleSlot: StoryObj = {
	render: () => ({
		components: { TabbedModal, ButtonStyled, SettingsIcon },
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
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Title Slot</button>
				</ButtonStyled>
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
		components: { TabbedModal, ButtonStyled },
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
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Footer</button>
				</ButtonStyled>
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

export const WithBadge: StoryObj = {
	render: () => ({
		components: { TabbedModal, ButtonStyled },
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
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Badge</button>
				</ButtonStyled>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const HiddenTabs: StoryObj = {
	render: () => ({
		components: { TabbedModal, ButtonStyled },
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
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Hidden Tab</button>
				</ButtonStyled>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const ManyTabs: StoryObj = {
	render: () => ({
		components: { TabbedModal, ButtonStyled },
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
					name: { id: 'language', defaultMessage: 'Language' },
					icon: LanguagesIcon,
					content: makeTabContent('Language'),
				},
				{
					name: { id: 'privacy', defaultMessage: 'Privacy' },
					icon: ShieldIcon,
					content: makeTabContent('Privacy'),
				},
				{
					name: { id: 'java', defaultMessage: 'Java and memory' },
					icon: CoffeeIcon,
					content: makeTabContent('Java and memory'),
				},
				{
					name: { id: 'instances', defaultMessage: 'Default instance options' },
					icon: GameIcon,
					content: makeTabContent('Default instance options'),
				},
				{
					name: { id: 'resources', defaultMessage: 'Resource management' },
					icon: GaugeIcon,
					content: makeTabContent('Resource management'),
				},
				{
					name: { id: 'window', defaultMessage: 'Window' },
					icon: MonitorIcon,
					content: makeTabContent('Window'),
				},
				{
					name: { id: 'hooks', defaultMessage: 'Launch hooks' },
					icon: WrenchIcon,
					content: makeTabContent('Launch hooks'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Many Tabs</button>
				</ButtonStyled>
				<TabbedModal ref="modalRef" header="Settings" :tabs="tabs" />
			</div>
		`,
	}),
}

export const WithTabProps: StoryObj = {
	render: () => ({
		components: { TabbedModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof TabbedModal> | null>(null)

			const PropsDisplay = defineComponent({
				props: { username: { type: String, default: '' }, role: { type: String, default: '' } },
				render() {
					return h('div', { class: 'space-y-2 py-2' }, [
						h('h2', { class: 'text-xl font-bold text-contrast m-0' }, 'User Profile'),
						h('p', { class: 'text-primary m-0' }, `Username: ${this.username}`),
						h('p', { class: 'text-primary m-0' }, `Role: ${this.role}`),
					])
				},
			})

			const tabs = [
				{
					name: { id: 'profile', defaultMessage: 'Profile' },
					icon: InfoIcon,
					content: PropsDisplay,
					props: { username: 'modrinth_user', role: 'Developer' },
				},
				{
					name: { id: 'settings', defaultMessage: 'Settings' },
					icon: PaintbrushIcon,
					content: makeTabContent('Settings'),
				},
			]
			return { modalRef, tabs }
		},
		template: /* html */ `
			<div>
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Tab Props</button>
				</ButtonStyled>
				<TabbedModal ref="modalRef" header="User" :tabs="tabs" />
			</div>
		`,
	}),
}

export const ScrollableContent: StoryObj = {
	render: () => ({
		components: { TabbedModal, ButtonStyled },
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
				<ButtonStyled color="brand">
					<button @click="modalRef?.show()">Open with Scrollable Content</button>
				</ButtonStyled>
				<TabbedModal ref="modalRef" header="Scrollable Demo" :tabs="tabs" />
			</div>
		`,
	}),
}
