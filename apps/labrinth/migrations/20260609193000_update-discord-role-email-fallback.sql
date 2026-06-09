UPDATE notifications_templates
SET plaintext_fallback = CONCAT(
	'Hey {user.name}!',
	CHR(10),
	CHR(10),
	'Your projects just passed 20,000 total downloads, nice!',
	CHR(10),
	CHR(10),
	'We want to invite you to Modrinth''s Creator Club, a space in our discord where you can chat with other creators, share feedback with us, and stay plugged in.',
	CHR(10),
	CHR(10),
	'To join just link your Discord account through Modrinth and we''ll grant access automatically!',
	CHR(10),
	CHR(10),
	'Join the Creator Club: {discord.link_url}'
)
WHERE channel = 'email'
	AND notification_type = 'discord_role_creator_club';
