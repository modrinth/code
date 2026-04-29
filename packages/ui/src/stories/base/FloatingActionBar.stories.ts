import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import FloatingActionBar from '../../components/base/FloatingActionBar.vue'
import { providePageContext } from '../../providers'

const meta = {
	title: 'Base/FloatingActionBar',
	component: FloatingActionBar,
} satisfies Meta<typeof FloatingActionBar>

export default meta
type Story = StoryObj<typeof meta>

function setupPageContext({
	leftOffset = '4rem',
	rightOffset = '0px',
	launcherPadding = 20,
	launcherWidth = 72,
} = {}) {
	const requestedLauncherPadding = ref<number | null>(null)

	providePageContext({
		hierarchicalSidebarAvailable: ref(true),
		showAds: ref(false),
		floatingActionBarOffsets: {
			left: ref(leftOffset),
			right: ref(rightOffset),
		},
		supportLauncher: {
			width: ref(launcherWidth),
			horizontalPadding: ref(launcherPadding),
			requestVerticalPadding: (_id, padding) => {
				requestedLauncherPadding.value = padding
			},
		},
		openExternalUrl: () => {},
	})

	return {
		launcherStyle: computed(() => ({
			right: `${launcherPadding}px`,
			bottom: `${requestedLauncherPadding.value ?? 20}px`,
			width: `${launcherWidth}px`,
			height: `${launcherWidth}px`,
		})),
		requestedLauncherPadding,
	}
}

export const Default: Story = {
	render: () => ({
		components: { ButtonStyled, FloatingActionBar },
		setup() {
			setupPageContext()
		},
		template: /*html*/ `
			<div class="h-32">
				<FloatingActionBar shown aria-label="Unsaved changes">
					<p class="m-0 text-sm font-semibold md:text-base">You have unsaved changes.</p>
					<div class="ml-auto flex gap-2">
						<ButtonStyled type="transparent">
							<button>Reset</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button>Save</button>
						</ButtonStyled>
					</div>
				</FloatingActionBar>
			</div>
		`,
	}),
}

export const WithSidebarOffset: Story = {
	render: () => ({
		components: { ButtonStyled, FloatingActionBar },
		setup() {
			setupPageContext({ rightOffset: '300px' })
		},
		template: /*html*/ `
			<div class="h-32">
				<FloatingActionBar shown aria-label="Selected files">
					<div class="flex items-center gap-0.5">
						<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">8 selected</span>
						<div class="mx-1 h-6 w-px bg-surface-5" />
						<ButtonStyled type="transparent">
							<button class="!text-primary">Clear</button>
						</ButtonStyled>
					</div>
					<div class="ml-auto flex items-center gap-0.5">
						<div class="mx-1 h-6 w-px bg-surface-5" />
						<ButtonStyled type="transparent" color="red" hover-color-fill="background">
							<button>Delete</button>
						</ButtonStyled>
					</div>
				</FloatingActionBar>
			</div>
		`,
	}),
}

export const AvoidingSupportLauncher: Story = {
	render: () => ({
		components: { ButtonStyled, FloatingActionBar },
		setup() {
			return setupPageContext({ rightOffset: '300px', launcherPadding: 320 })
		},
		template: /*html*/ `
			<div class="h-40">
				<div
					class="fixed rounded-full bg-blue shadow-lg"
					:style="launcherStyle"
				/>
				<FloatingActionBar shown aria-label="Selected files">
					<div class="flex items-center gap-0.5">
						<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">8 selected</span>
						<div class="mx-1 h-6 w-px bg-surface-5" />
						<ButtonStyled type="transparent">
							<button class="!text-primary">Clear</button>
						</ButtonStyled>
					</div>
					<div class="ml-auto flex items-center gap-0.5">
						<div class="mx-1 h-6 w-px bg-surface-5" />
						<ButtonStyled type="transparent" color="red" hover-color-fill="background">
							<button>Delete</button>
						</ButtonStyled>
					</div>
				</FloatingActionBar>
			</div>
		`,
	}),
}
