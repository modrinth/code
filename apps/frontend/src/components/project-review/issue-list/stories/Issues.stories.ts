import { Input } from '@modrinth/ui'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

import { createIssueCatalogue, provideIssueCatalogueProject } from './issue-catalogue'
import IssueCatalogueRow from './IssueCatalogueRow.vue'

const meta = {
	title: 'Website/Moderation/Issues',
	parameters: { layout: 'fullscreen', controls: { disable: true } },
	beforeEach: () => {
		const previousAuth = Object.getOwnPropertyDescriptor(globalThis, 'useAuthState')
		Object.defineProperty(globalThis, 'useAuthState', {
			configurable: true,
			value: () => ref({ user: null }),
		})
		return () => {
			if (previousAuth) Object.defineProperty(globalThis, 'useAuthState', previousAuth)
			else Reflect.deleteProperty(globalThis, 'useAuthState')
		}
	},
	render: () => ({
		components: { Input, IssueCatalogueRow },
		setup() {
			provideIssueCatalogueProject()
			const search = ref('')
			const catalogue = createIssueCatalogue()
			const issues = catalogue.availableIssues
			const visibleIds = computed(
				() =>
					new Set(
						issues.value
							.filter((issue) =>
								`${issue.category} ${issue.title} ${issue.id}`
									.toLowerCase()
									.includes(search.value.toLowerCase()),
							)
							.map(({ id }) => id),
					),
			)
			return { search, issues, visibleIds }
		},
		template: /* html */ `
			<div class="mx-auto flex flex-col gap-6 p-6 w-fit" style="max-width: 1500px">
				<header>
					<h1 class="m-0 text-2xl font-bold text-contrast">Moderation issues</h1>
					<Input v-model="search" type="search" aria-label="Filter issues" placeholder="Filter by title, category, or ID…" class="w-full" />
					<p class="text-sm text-secondary">{{ visibleIds.size }} of {{ issues.length }} issues</p>
				</header>
				<IssueCatalogueRow v-for="issue in issues" v-show="visibleIds.has(issue.id)" :key="issue.id" :issue-id="issue.id" />
				<p v-if="!visibleIds.size">No matching issues.</p>
			</div>
		`,
	}),
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const AllIssues: Story = {}
