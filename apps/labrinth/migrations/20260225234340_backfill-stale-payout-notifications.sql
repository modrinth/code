-- Mark all stale payouts_values_notifications rows as notified.
-- These accumulated due to a bug where a FK constraint violation in
-- notifications_deliveries caused the entire transaction to roll back,
-- leaving notified = FALSE permanently. Without this, fixing the bug
-- would cause the service to spam ancient payout notifications.
UPDATE payouts_values_notifications
SET notified = TRUE
WHERE notified = FALSE
  AND date_available <= NOW();
