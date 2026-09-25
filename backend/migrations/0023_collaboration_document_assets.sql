CREATE TABLE mahfel.collaboration_document_assets (
    document_id UUID NOT NULL REFERENCES mahfel.collaboration_documents (id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES mahfel.files (id) ON DELETE RESTRICT,
    PRIMARY KEY (document_id, file_id)
);

CREATE INDEX collaboration_document_assets_file_idx
ON mahfel.collaboration_document_assets (file_id);

CREATE FUNCTION mahfel.protect_collaboration_asset() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'deleting' AND OLD.status <> 'deleting'
        AND EXISTS (
            SELECT 1 FROM mahfel.collaboration_document_assets WHERE file_id = OLD.id
        )
    THEN
        RAISE EXCEPTION 'file is referenced by a collaboration document' USING ERRCODE = '23505';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER protect_collaboration_asset_before_delete
BEFORE UPDATE OF status ON mahfel.files
FOR EACH ROW EXECUTE FUNCTION mahfel.protect_collaboration_asset();
