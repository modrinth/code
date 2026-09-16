/** Default brand mark when the user has no uploaded avatar. */
export const OWYX_AVATAR_FALLBACK = "/owyx-avatar-fallback.svg";

/** Prefer uploaded avatar; otherwise brand favicon SVG (never a letter glyph). */
export function resolveSiteAvatarUrl(avatarUrl?: string | null): string {
  const raw = (avatarUrl ?? "").trim();
  if (!raw) return OWYX_AVATAR_FALLBACK;
  return raw;
}
