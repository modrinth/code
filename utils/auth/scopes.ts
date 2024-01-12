const scopeDefinitions = [
  {
    name: 'USER_READ_EMAIL',
    description: 'Read your email',
    label: 'Read user email',
    value: BigInt(1) << BigInt(0),
  },
  {
    name: 'USER_READ',
    description: 'Access your public profile information',
    label: 'Read user data',
    value: BigInt(1) << BigInt(1),
  },
  {
    name: 'USER_WRITE',
    description: 'Write to your profile',
    label: 'Write user data',
    value: BigInt(1) << BigInt(2),
  },
  {
    name: 'USER_DELETE',
    description: 'Delete your account',
    label: 'Delete your account',
    value: BigInt(1) << BigInt(3),
  },
  {
    name: 'USER_AUTH_WRITE',
    description: 'Modify your authentication data',
    label: 'Write auth data',
    value: BigInt(1) << BigInt(4),
  },
  {
    name: 'NOTIFICATION_READ',
    description: 'Read your notifications',
    label: 'Read notifications',
    value: BigInt(1) << BigInt(5),
  },
  {
    name: 'NOTIFICATION_WRITE',
    description: 'Delete/View your notifications',
    label: 'Write notifications',
    value: BigInt(1) << BigInt(6),
  },
  {
    name: 'PAYOUTS_READ',
    description: 'Read your payouts data',
    label: 'Read payouts',
    value: BigInt(1) << BigInt(7),
  },
  {
    name: 'PAYOUTS_WRITE',
    description: 'Withdraw money',
    label: 'Write payouts',
    value: BigInt(1) << BigInt(8),
  },
  {
    name: 'ANALYTICS',
    description: 'Access your analytics data',
    label: 'Read analytics',
    value: BigInt(1) << BigInt(9),
  },
  {
    name: 'PROJECT_CREATE',
    description: 'Create new projects',
    label: 'Create projects',
    value: BigInt(1) << BigInt(10),
  },
  {
    name: 'PROJECT_READ',
    description: 'Read all your projects',
    label: 'Read projects',
    value: BigInt(1) << BigInt(11),
  },
  {
    name: 'PROJECT_WRITE',
    description: 'Write to project data',
    label: 'Write projects',
    value: BigInt(1) << BigInt(12),
  },
  {
    name: 'PROJECT_DELETE',
    description: 'Delete your projects',
    label: 'Delete projects',
    value: BigInt(1) << BigInt(13),
  },
  {
    name: 'VERSION_CREATE',
    description: 'Create new versions',
    label: 'Create versions',
    value: BigInt(1) << BigInt(14),
  },
  {
    name: 'VERSION_READ',
    description: 'Read all versions',
    label: 'Read versions',
    value: BigInt(1) << BigInt(15),
  },
  {
    name: 'VERSION_WRITE',
    description: 'Write to version data',
    label: 'Write versions',
    value: BigInt(1) << BigInt(16),
  },
  {
    name: 'VERSION_DELETE',
    description: 'Delete a version',
    label: 'Delete versions',
    value: BigInt(1) << BigInt(17),
  },
  {
    name: 'REPORT_CREATE',
    description: 'Create reports',
    label: 'Create reports',
    value: BigInt(1) << BigInt(18),
  },
  {
    name: 'REPORT_READ',
    description: 'Read reports',
    label: 'Read reports',
    value: BigInt(1) << BigInt(19),
  },
  {
    name: 'REPORT_WRITE',
    description: 'Edit reports',
    label: 'Write reports',
    value: BigInt(1) << BigInt(20),
  },
  {
    name: 'REPORT_DELETE',
    description: 'Delete reports',
    label: 'Delete reports',
    value: BigInt(1) << BigInt(21),
  },
  {
    name: 'THREAD_READ',
    description: 'Read threads',
    label: 'Read threads',
    value: BigInt(1) << BigInt(22),
  },
  {
    name: 'THREAD_WRITE',
    description: 'Write to threads',
    label: 'Write threads',
    value: BigInt(1) << BigInt(23),
  },
  {
    name: 'PAT_CREATE',
    description: 'Create personal API tokens',
    label: 'Create PATs',
    value: BigInt(1) << BigInt(24),
  },
  {
    name: 'PAT_READ',
    description: 'View created API tokens',
    label: 'Read PATs',
    value: BigInt(1) << BigInt(25),
  },
  {
    name: 'PAT_WRITE',
    description: 'Edit personal API tokens',
    label: 'Write PATs',
    value: BigInt(1) << BigInt(26),
  },
  {
    name: 'PAT_DELETE',
    description: 'Delete your personal API tokens',
    label: 'Delete PATs',
    value: BigInt(1) << BigInt(27),
  },
  {
    name: 'SESSION_READ',
    description: 'Read active sessions',
    label: 'Read sessions',
    value: BigInt(1) << BigInt(28),
  },
  {
    name: 'SESSION_DELETE',
    description: 'Delete sessions',
    label: 'Delete sessions',
    value: BigInt(1) << BigInt(29),
  },
  {
    name: 'PERFORM_ANALYTICS',
    description: 'Perform analytics actions',
    label: 'Perform analytics',
    value: BigInt(1) << BigInt(30),
  },
  {
    name: 'COLLECTION_CREATE',
    description: 'Create collections',
    label: 'Create collections',
    value: BigInt(1) << BigInt(31),
  },
  {
    name: 'COLLECTION_READ',
    description: 'Read collections',
    label: 'Read collections',
    value: BigInt(1) << BigInt(32),
  },
  {
    name: 'COLLECTION_WRITE',
    description: 'Write to collections',
    label: 'Write collections',
    value: BigInt(1) << BigInt(33),
  },
  {
    name: 'COLLECTION_DELETE',
    description: 'Delete collections',
    label: 'Delete collections',
    value: BigInt(1) << BigInt(34),
  },
  {
    name: 'ORGANIZATION_CREATE',
    description: 'Create organizations',
    label: 'Create organizations',
    value: BigInt(1) << BigInt(35),
  },
  {
    name: 'ORGANIZATION_READ',
    description: 'Read organizations',
    label: 'Read organizations',
    value: BigInt(1) << BigInt(36),
  },
  {
    name: 'ORGANIZATION_WRITE',
    description: 'Write to organizations',
    label: 'Write organizations',
    value: BigInt(1) << BigInt(37),
  },
  {
    name: 'ORGANIZATION_DELETE',
    description: 'Delete organizations',
    label: 'Delete organizations',
    value: BigInt(1) << BigInt(38),
  },
  {
    name: 'SESSION_ACCESS',
    description: 'Access modrinth-issued sessions',
    label: 'Access sessions',
    value: BigInt(1) << BigInt(39),
  },
]

const Scopes = scopeDefinitions.reduce((acc, scope) => {
  acc[scope.name] = scope.value
  return acc
}, {} as Record<string, bigint>)

const ScopeLabels = scopeDefinitions.reduce((acc, scope) => {
  acc[scope.name] = scope.label
  return acc
}, {} as Record<string, string>)

const ScopeDescriptions = scopeDefinitions.reduce((acc, scope) => {
  acc[scope.name] = scope.description
  return acc
}, {} as Record<string, string>)

export const getScopeLabel = (scope: string | bigint) => {
  if (typeof scope === 'bigint') {
    for (const [scopeName, scopeFlag] of Object.entries(Scopes)) {
      if (scopeFlag === scope) {
        scope = scopeName
        break
      }
    }

    if (typeof scope === 'bigint') {
      return 'Unknown scope'
    }
  }

  return ScopeLabels?.[scope] ?? 'Unknown scope'
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
