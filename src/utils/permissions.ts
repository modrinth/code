export class Permissions {
	uploadVersions = false
	deleteVersion = false
	editDetails = false
	editBody = false
	manageInvites = false
	removeMember = false
	editMember = false
	deleteProject = false

	get settingsPage(): boolean {
		return this.manageInvites || this.removeMember || this.editMember || this.deleteProject
	}

	toInteger(): number {
		return (
			(this.uploadVersions ? 1 : 0) |
			(this.deleteVersion ? 1 << 1 : 0) |
			(this.editDetails ? 1 << 2 : 0) |
			(this.editBody ? 1 << 3 : 0) |
			(this.manageInvites ? 1 << 4 : 0) |
			(this.removeMember ? 1 << 5 : 0) |
			(this.editMember ? 1 << 6 : 0) |
			(this.deleteProject ? 1 << 7 : 0)
		)
	}

	constructor(from: number | 'ALL' | null) {
		if (from === 'ALL' || from === 0b11111111 || from === null) {
			Object.keys(this).forEach((v) => (this[v] = true))
		} else if (typeof from === 'number') {
			this.uploadVersions = !!(from & (1 << 0))
			this.deleteVersion = !!(from & (1 << 1))
			this.editDetails = !!(from & (1 << 2))
			this.editBody = !!(from & (1 << 3))
			this.manageInvites = !!(from & (1 << 4))
			this.removeMember = !!(from & (1 << 5))
			this.editMember = !!(from & (1 << 6))
			this.deleteProject = !!(from & (1 << 7))
		}
	}
}
