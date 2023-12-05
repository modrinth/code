export const Scopes = {
  USER_READ_EMAIL: BigInt(1) << BigInt(0),
  USER_READ: BigInt(1) << BigInt(1),
  USER_WRITE: BigInt(1) << BigInt(2),
  USER_DELETE: BigInt(1) << BigInt(3),
  USER_AUTH_WRITE: BigInt(1) << BigInt(4),
  NOTIFICATION_READ: BigInt(1) << BigInt(5),
  NOTIFICATION_WRITE: BigInt(1) << BigInt(6),
  PAYOUTS_READ: BigInt(1) << BigInt(7),
  PAYOUTS_WRITE: BigInt(1) << BigInt(8),
  ANALYTICS: BigInt(1) << BigInt(9),
  PROJECT_CREATE: BigInt(1) << BigInt(10),
  PROJECT_READ: BigInt(1) << BigInt(11),
  PROJECT_WRITE: BigInt(1) << BigInt(12),
  PROJECT_DELETE: BigInt(1) << BigInt(13),
  VERSION_CREATE: BigInt(1) << BigInt(14),
  VERSION_READ: BigInt(1) << BigInt(15),
  VERSION_WRITE: BigInt(1) << BigInt(16),
  VERSION_DELETE: BigInt(1) << BigInt(17),
  REPORT_CREATE: BigInt(1) << BigInt(18),
  REPORT_READ: BigInt(1) << BigInt(19),
  REPORT_WRITE: BigInt(1) << BigInt(20),
  REPORT_DELETE: BigInt(1) << BigInt(21),
  THREAD_READ: BigInt(1) << BigInt(22),
  THREAD_WRITE: BigInt(1) << BigInt(23),
  PAT_CREATE: BigInt(1) << BigInt(24),
  PAT_READ: BigInt(1) << BigInt(25),
  PAT_WRITE: BigInt(1) << BigInt(26),
  PAT_DELETE: BigInt(1) << BigInt(27),
  SESSION_READ: BigInt(1) << BigInt(28),
  SESSION_DELETE: BigInt(1) << BigInt(29),
  PERFORM_ANALYTICS: BigInt(1) << BigInt(30),
  COLLECTION_CREATE: BigInt(1) << BigInt(31),
  COLLECTION_READ: BigInt(1) << BigInt(32),
  COLLECTION_WRITE: BigInt(1) << BigInt(33),
  COLLECTION_DELETE: BigInt(1) << BigInt(34),
  ORGANIZATION_CREATE: BigInt(1) << BigInt(35),
  ORGANIZATION_READ: BigInt(1) << BigInt(36),
  ORGANIZATION_WRITE: BigInt(1) << BigInt(37),
  ORGANIZATION_DELETE: BigInt(1) << BigInt(38),
  SESSION_ACCESS: BigInt(1) << BigInt(39),
}

export const restrictedScopes = [
  Scopes.PAT_READ,
  Scopes.PAT_CREATE,
  Scopes.PAT_WRITE,
  Scopes.PAT_DELETE,
  Scopes.SESSION_READ,
  Scopes.SESSION_DELETE,
  Scopes.SESSION_ACCESS,
  Scopes.USER_AUTH_WRITE,
  Scopes.USER_DELETE,
  Scopes.PERFORM_ANALYTICS,
]

export const scopeList = Object.entries(Scopes)
  .filter(([_, value]) => !restrictedScopes.includes(value))
  .map(([key, _]) => key)

export const encodeScopes = (scopes: string[]) => {
  let scopeFlag = BigInt(0)

  // We iterate over the provided scopes
  for (const scope of scopes) {
    // We iterate over the entries of the Scopes object
    for (const [scopeName, scopeFlagValue] of Object.entries(Scopes)) {
      // If the scope name is the same as the provided scope, add the scope flag to the scopeFlag variable
      if (scopeName === scope) {
        scopeFlag = scopeFlag | scopeFlagValue
      }
    }
  }

  return scopeFlag
}

export const decodeScopes = (scopes: bigint | number) => {
  if (typeof scopes === 'number') {
    scopes = BigInt(scopes)
  }

  const authorizedScopes = []

  // We iterate over the entries of the Scopes object
  for (const [scopeName, scopeFlag] of Object.entries(Scopes)) {
    // If the scope flag is present in the provided number, add the scope name to the list
    if ((scopes & scopeFlag) === scopeFlag) {
      authorizedScopes.push(scopeName)
    }
  }

  return authorizedScopes
}

export const hasScope = (scopes: bigint, scope: string) => {
  const authorizedScopes = decodeScopes(scopes)
  return authorizedScopes.includes(scope)
}

export const toggleScope = (scopes: bigint, scope: string) => {
  const authorizedScopes = decodeScopes(scopes)
  if (authorizedScopes.includes(scope)) {
    return encodeScopes(authorizedScopes.filter((authorizedScope) => authorizedScope !== scope))
  } else {
    return encodeScopes([...authorizedScopes, scope])
  }
}

export const getScopeDefinitions = (scopes: bigint) => {
  return decodeScopes(scopes)
    .filter((scope) => Object.keys(ScopeDescriptions).includes(scope))
    .map((scope) => (ScopeDescriptions as Record<string, string>)[scope])
}

export const ScopeDescriptions = {
  USER_READ_EMAIL: 'Read your email',
  USER_READ: 'Access your public profile information',
  USER_WRITE: 'Write to your profile',
  USER_DELETE: 'Delete your account',
  USER_AUTH_WRITE: 'Modify your authentication data',
  NOTIFICATION_READ: 'Read your notifications',
  NOTIFICATION_WRITE: 'Delete/View your notifications',
  PAYOUTS_READ: 'Read your payouts data',
  PAYOUTS_WRITE: 'Withdraw money',
  ANALYTICS: 'Access your analytics data',
  PROJECT_CREATE: 'Create new projects',
  PROJECT_READ: 'Read all your projects',
  PROJECT_WRITE: 'Write to project data',
  PROJECT_DELETE: 'Delete your projects',
  VERSION_CREATE: 'Create new versions',
  VERSION_READ: 'Read all versions',
  VERSION_WRITE: 'Write to version data',
  VERSION_DELETE: 'Delete a version',
  REPORT_CREATE: 'Create reports',
  REPORT_READ: 'Read reports',
  REPORT_WRITE: 'Edit reports',
  REPORT_DELETE: 'Delete reports',
  THREAD_READ: 'Read threads',
  THREAD_WRITE: 'Write to threads',
  PAT_CREATE: 'Create personal API tokens',
  PAT_READ: 'View created API tokens',
  PAT_WRITE: 'Edit personal API tokens',
  PAT_DELETE: 'Delete your personal API tokens',
  SESSION_READ: 'Read active sessions',
  SESSION_DELETE: 'Delete sessions',
  PERFORM_ANALYTICS: 'Perform analytics actions',
  COLLECTION_CREATE: 'Create collections',
  COLLECTION_READ: 'Read collections',
  COLLECTION_WRITE: 'Write to collections',
  COLLECTION_DELETE: 'Delete collections',
  ORGANIZATION_CREATE: 'Create organizations',
  ORGANIZATION_READ: 'Read organizations',
  ORGANIZATION_WRITE: 'Write to organizations',
  ORGANIZATION_DELETE: 'Delete organizations',
  SESSION_ACCESS: 'Access modrinth-issued sessions',
}
