ALTER TABLE mahfel.sessions
ADD COLUMN title TEXT NULL CHECK (title IS NULL OR char_length(title) BETWEEN 1 AND 300),
ADD COLUMN scheduled_duration_minutes INTEGER NULL
CHECK (scheduled_duration_minutes IS NULL OR scheduled_duration_minutes BETWEEN 1 AND 10080);
