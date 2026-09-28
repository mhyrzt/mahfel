ALTER TABLE mahfel.collaboration_edit_grants
ADD COLUMN can_publish_audio BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN can_publish_video BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN can_share_screen BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN can_present BOOLEAN NOT NULL DEFAULT false;

UPDATE mahfel.collaboration_edit_grants SET can_present = can_edit;
