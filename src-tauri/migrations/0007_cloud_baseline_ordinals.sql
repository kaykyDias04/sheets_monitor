-- Remote version labels are immutable; version remains a device-local unique ordinal.
ALTER TABLE baseline_versions ADD COLUMN sync_version INTEGER CHECK(sync_version IS NULL OR sync_version >= 1);
