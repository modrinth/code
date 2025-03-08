export const isPermission = (perms?: number, bitflag?: number) => {
  if (!perms || !bitflag) return false
  return (perms & bitflag) === bitflag
}
