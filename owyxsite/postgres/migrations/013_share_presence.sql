-- Social privacy: allow users to hide online presence from friends.
-- Also applied idempotently by ensureFriendsSchema() in friends.js.

ALTER TABLE public.user_social_settings
  ADD COLUMN IF NOT EXISTS share_presence BOOLEAN NOT NULL DEFAULT true;

COMMENT ON COLUMN public.user_social_settings.share_presence IS
  'When false, presence heartbeats are ignored and friends always see offline.';
