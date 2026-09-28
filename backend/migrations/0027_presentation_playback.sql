ALTER TABLE mahfel.collaboration_presentations
ADD COLUMN playback_position_ms BIGINT NULL
CHECK (playback_position_ms IS NULL OR playback_position_ms >= 0),
ADD COLUMN playback_paused BOOLEAN NULL,
ADD COLUMN playback_rate DOUBLE PRECISION NULL
CHECK (playback_rate IS NULL OR (playback_rate >= 0.25 AND playback_rate <= 4));
