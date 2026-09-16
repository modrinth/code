import type { SplitviewApi } from 'dockview-vue'
import { nextTick, onBeforeUnmount, onMounted, shallowRef } from 'vue'

const duration = 200
const transitionClass = 'project-review-transition-column'
const properties = ['--review-column-width', '--review-column-left']

type Geometry = { left: number; width: number }
type Transition = {
	views: HTMLElement[]
	center: HTMLElement
	current: Geometry
}

export function useSidebarTransition() {
	const centerElement = shallowRef<HTMLElement | null>(null)
	let transition: Transition | undefined
	let frame: number | undefined
	let revision = 0
	const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')

	function finishSidebarTransition() {
		revision++
		if (frame !== undefined) cancelAnimationFrame(frame)
		frame = undefined
		for (const view of transition?.views ?? []) {
			view.classList.remove(transitionClass)
			for (const property of properties) view.style.removeProperty(property)
		}
		transition = undefined
	}

	function setGeometry(view: HTMLElement, geometry: Geometry) {
		view.style.setProperty('--review-column-width', `${geometry.width}px`)
		view.style.setProperty('--review-column-left', `${geometry.left}px`)
		view.classList.add(transitionClass)
	}

	async function transitionSidebar(api: SplitviewApi, side: 'left' | 'right', visible: boolean) {
		const center = centerElement.value?.closest<HTMLElement>('.dv-view')
		const container = center?.parentElement
		const columns = center?.closest<HTMLElement>('.project-review-columns')
		if (!center || !container || !columns || reducedMotion.matches) {
			finishSidebarTransition()
			api.getPanel(side)?.api.setVisible(visible)
			return
		}

		if (frame !== undefined) cancelAnimationFrame(frame)
		frame = undefined
		const currentRevision = ++revision
		const viewport = columns
		const views = Array.from(container.children).filter(
			(view): view is HTMLElement => view instanceof HTMLElement,
		)
		const containerLeft = container.getBoundingClientRect().left
		const before = views.map((view) => ({
			left: view.getBoundingClientRect().left - containerLeft,
			width: view.getBoundingClientRect().width,
		}))
		const current = transition?.current ?? {
			left: center.getBoundingClientRect().left - containerLeft,
			width: center.getBoundingClientRect().width,
		}
		const active: Transition = transition ?? {
			views,
			center,
			current,
		}
		transition = active
		views.forEach((view, index) => setGeometry(view, before[index]))
		api.getPanel(side)?.api.setVisible(visible)
		await nextTick()
		if (currentRevision !== revision) return

		const width = viewport.clientWidth
		const height = viewport.clientHeight
		api.layout(width, height)
		const target = {
			left: Number.parseFloat(center.style.left) || 0,
			width: api.getPanel('center')?.api.width ?? current.width,
		}
		views.forEach((view, index) => {
			if (view === center) return
			const width = Number.parseFloat(view.style.width) || 0
			setGeometry(view, {
				left: width > 0 ? Number.parseFloat(view.style.left) || 0 : before[index].left,
				width: Math.max(before[index].width, width),
			})
		})

		function render(geometry: Geometry) {
			active.current = geometry
			setGeometry(active.center, geometry)
		}

		render(current)
		const started = performance.now()
		function tick(now: number) {
			if (currentRevision !== revision) return
			if (viewport.clientWidth !== width || viewport.clientHeight !== height) {
				finishSidebarTransition()
				return
			}
			const progress = Math.min(1, (now - started) / duration)
			const eased = 1 - (1 - progress) ** 3
			render({
				left: current.left + (target.left - current.left) * eased,
				width: current.width + (target.width - current.width) * eased,
			})
			if (progress < 1) frame = requestAnimationFrame(tick)
			else finishSidebarTransition()
		}
		frame = requestAnimationFrame(tick)
	}

	onMounted(() => {
		window.addEventListener('resize', finishSidebarTransition)
		window.addEventListener('pagehide', finishSidebarTransition)
		reducedMotion.addEventListener('change', finishSidebarTransition)
	})
	onBeforeUnmount(() => {
		finishSidebarTransition()
		window.removeEventListener('resize', finishSidebarTransition)
		window.removeEventListener('pagehide', finishSidebarTransition)
		reducedMotion.removeEventListener('change', finishSidebarTransition)
	})

	return { centerElement, transitionSidebar, finishSidebarTransition }
}
