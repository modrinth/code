export const isPermission = (perms?: number | null, bitflag?: number | null) => {
	if (!perms || !bitflag) return false
	return (perms & bitflag) === bitflag
}
