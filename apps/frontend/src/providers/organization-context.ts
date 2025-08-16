import { createContext } from '@modrinth/ui'
import type { Organization, OrganizationMember, ProjectV3 } from '@modrinth/utils'

export class OrganizationContext {
	public readonly organization: Ref<Organization | null>
	public readonly projects: Ref<ProjectV3[] | null>
	private readonly auth: Ref<any>
	private readonly tags: Ref<any>
	private readonly refreshFunction: () => Promise<void>

	public constructor(
		organization: Ref<Organization | null>,
		projects: Ref<ProjectV3[] | null>,
		auth: Ref<any>,
		tags: Ref<any>,
		refreshFunction: () => Promise<void>,
	) {
		this.organization = organization
		this.projects = projects
		this.auth = auth
		this.tags = tags
		this.refreshFunction = refreshFunction
	}

	public refresh = async () => {
		if (this.organization.value === null) {
			throw new Error('Organization is not set.')
		}

		await this.refreshFunction()
	}

	public currentMember = computed<Partial<OrganizationMember> | null>(() => {
		if (this.auth.value.user && this.organization.value) {
			const member = this.organization.value.members.find(
				(x) => x.user.id === this.auth.value.user.id,
			)

			if (member) {
				return member
			}

			if (this.tags.value.staffRoles.includes(this.auth.value.user.role)) {
				return {
					user: this.auth.value.user,
					role: this.auth.value.user.role,
					permissions: this.auth.value.user.role === 'admin' ? 1023 : 12,
					accepted: true,
					payouts_split: 0,
					avatar_url: this.auth.value.user.avatar_url,
					name: this.auth.value.user.username,
				} as Partial<OrganizationMember>
			}
		}

		return null
	})

	public hasPermission = computed(() => {
		const EDIT_DETAILS = 1 << 2
		return (
			this.currentMember.value &&
			(this.currentMember.value.permissions! & EDIT_DETAILS) === EDIT_DETAILS
		)
	})

	public patchIcon = async (icon: { name: string }) => {
		if (this.organization.value === null) {
			throw new Error('Organization is not set.')
		}

		const ext = icon.name.split('.').pop()
		await useBaseFetch(`organization/${this.organization.value.id}/icon`, {
			method: 'PATCH',
			body: icon,
			query: { ext },
			apiVersion: 3,
		})
	}

	public deleteIcon = async () => {
		if (this.organization.value === null) {
			throw new Error('Organization is not set.')
		}

		await useBaseFetch(`organization/${this.organization.value.id}/icon`, {
			method: 'DELETE',
			apiVersion: 3,
		})
	}

	public patchOrganization = async (
		newData: Partial<{ slug: string; name: string; description: string }>,
	) => {
		if (this.organization.value === null) {
			throw new Error('Organization is not set.')
		}

		await useBaseFetch(`organization/${this.organization.value.id}`, {
			method: 'PATCH',
			body: newData,
			apiVersion: 3,
		})

		await this.refreshFunction()
	}
}

export const [injectOrganizationContext, provideOrganizationContext] =
	createContext<OrganizationContext>('[id].vue', 'organizationContext')
