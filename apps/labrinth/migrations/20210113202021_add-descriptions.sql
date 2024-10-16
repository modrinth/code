ALTER TABLE mods
    ADD COLUMN body varchar(65536) NOT NULL DEFAULT '';
ALTER TABLE mods
    ALTER COLUMN body_url DROP NOT NULL;
ALTER TABLE versions
    ADD COLUMN changelog varchar(65536) NOT NULL DEFAULT '';

INSERT INTO users (
    id, github_id, username, name, email,
    avatar_url, bio, created
)
VALUES (
           127155982985829, 10137, 'Ghost', NULL, NULL,
           'https://avatars2.githubusercontent.com/u/10137', 'A deleted user', NOW()
       );