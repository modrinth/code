-- mistake from previous migration `20260218183440_server_project_links_categories`
DELETE FROM categories
WHERE
    header = 'minecraft_server_community'
    AND category = 'adventure-mode';
