export class Permissions {
	data = {
		uploadVersions: false,
		deleteVersion: false,
		editDetails: false,
		editBody: false,
		manageInvites: false,
		removeMember: false,
		editMember: false,
		deleteProject: false,
	};

	get settingsPage(): boolean {
		return (
			this.data.manageInvites ||
			this.data.removeMember ||
			this.data.editMember ||
			this.data.deleteProject
		);
	}

	constructor(from: number | 'ALL' | null) {
		if (from === 'ALL' || from === 0b11111111 || from === null) {
			Object.keys(this.data).forEach((v) => (this.data[v] = true));
		} else if (typeof from === 'number') {
			this.data = {
				uploadVersions: !!(from & (1 << 0)),
				deleteVersion: !!(from & (1 << 1)),
				editDetails: !!(from & (1 << 2)),
				editBody: !!(from & (1 << 3)),
				manageInvites: !!(from & (1 << 4)),
				removeMember: !!(from & (1 << 5)),
				editMember: !!(from & (1 << 6)),
				deleteProject: !!(from & (1 << 7)),
			};
		}
	}
}
