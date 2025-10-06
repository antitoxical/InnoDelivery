-- This file should undo anything in [up.sql](cci:7://file:///C:/Users/Danila/RustroverProjects/InnoDelivery/user_service/migrations/2025-08-20-134547_create_users_and_couriers/up.sql:0:0-0:0)
ALTER TABLE couriers
DROP COLUMN is_blocked,
DROP COLUMN is_deleted;
