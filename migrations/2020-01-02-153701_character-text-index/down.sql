-- This file should undo anything in `up.sql`
DROP TRIGGER "character_search_text_update_trigger" ON "character";
DROP FUNCTION "character_search_text_update";
DROP INDEX "character_search_text_index";
ALTER TABLE "character" DROP COLUMN "search_text";