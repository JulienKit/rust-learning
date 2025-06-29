-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
    VALUES (
            gen_random_uuid(),
            'admin',
            '$argon2id$v=19$m=15000,t=2,p=1$EcyHkU/bb6B6Gem3xjP1DQ$ewa99U0IRb9fr4wzJympkDTunL52Fakz2JQrEG31S9Y'
           )