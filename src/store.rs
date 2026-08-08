use std::{
    error::Error,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::{
    compile::verify_compiled_source,
    evidence::CompiledSource,
    source::{CanonicalSource, SourceProvider},
};

// Version 2 segments captions into passages. Version 1 entries hold fragment-level
// evidence and must be reacquired rather than served.
const STORE_SCHEMA_VERSION: u32 = 2;
const MAX_POINTER_BYTES: u64 = 128;
const MAX_SOURCE_BYTES: u64 = 100 * 1024 * 1024;
const VERSION_PREFIX: &str = "source-v1:sha256:";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveOutcome {
    NewVersion,
    ReusedVersion,
}

#[derive(Debug)]
pub enum StoreError {
    Io {
        stage: &'static str,
        error: io::Error,
    },
    InvalidPointer,
    InvalidStoredSource,
    UnsupportedSchema(u32),
    Serialise(serde_json::Error),
    Persist(&'static str),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { stage, error } => write!(formatter, "{stage} failed: {error}"),
            Self::InvalidPointer => formatter.write_str("cache pointer is invalid"),
            Self::InvalidStoredSource => {
                formatter.write_str("cached source failed integrity validation")
            }
            Self::UnsupportedSchema(version) => {
                write!(formatter, "cache schema version {version} is unsupported")
            }
            Self::Serialise(error) => {
                write!(formatter, "serialising cached source failed: {error}")
            }
            Self::Persist(artifact) => write!(formatter, "persisting cache {artifact} failed"),
        }
    }
}

impl Error for StoreError {}

#[derive(Clone, Debug)]
pub struct FileSourceStore {
    root: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredSource {
    schema_version: u32,
    compiled: CompiledSource,
}

impl FileSourceStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Loads the latest validated compiled source for a language or default track.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when a pointer, file, future schema or compiled
    /// evidence record fails validation. Known legacy entries are treated as a
    /// cache miss so the provider can reacquire them.
    pub fn load_latest(
        &self,
        source: &CanonicalSource,
        language: Option<&str>,
    ) -> Result<Option<CompiledSource>, StoreError> {
        let pointer_key = pointer_key(language)?;
        let source_dir = self.source_dir(source);
        let pointer_path = source_dir
            .join("tracks")
            .join(format!("{pointer_key}.latest"));
        let pointer = match read_limited(&pointer_path, MAX_POINTER_BYTES) {
            Ok(bytes) => bytes,
            Err(StoreError::Io { error, .. }) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let digest = std::str::from_utf8(&pointer)
            .ok()
            .map(str::trim)
            .filter(|value| is_digest(value))
            .ok_or(StoreError::InvalidPointer)?;
        let version_path = source_dir.join("versions").join(format!("{digest}.json"));
        let stored_bytes = read_limited(&version_path, MAX_SOURCE_BYTES)?;
        let stored: StoredSource =
            serde_json::from_slice(&stored_bytes).map_err(|_| StoreError::InvalidStoredSource)?;
        match stored.schema_version {
            STORE_SCHEMA_VERSION => {}
            1 => return Ok(None),
            version => return Err(StoreError::UnsupportedSchema(version)),
        }
        verify_stored(&stored.compiled, source, language, digest)?;
        Ok(Some(stored.compiled))
    }

    /// Persists an immutable compiled version and atomically advances track pointers.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the compiled source is invalid or local files
    /// cannot be created, verified or atomically persisted.
    pub fn save(
        &self,
        compiled: &CompiledSource,
        set_as_default: bool,
    ) -> Result<SaveOutcome, StoreError> {
        verify_compiled_source(compiled).map_err(|_| StoreError::InvalidStoredSource)?;
        let digest =
            version_digest(&compiled.source_version).ok_or(StoreError::InvalidStoredSource)?;
        let source_dir = self.source_dir(&compiled.source);
        let versions_dir = source_dir.join("versions");
        let tracks_dir = source_dir.join("tracks");
        create_private_directory(&versions_dir)?;
        create_private_directory(&tracks_dir)?;

        let version_path = versions_dir.join(format!("{digest}.json"));
        let outcome = match read_stored_source(&version_path) {
            Ok(stored) => match stored.schema_version {
                STORE_SCHEMA_VERSION if stored.compiled == *compiled => SaveOutcome::ReusedVersion,
                STORE_SCHEMA_VERSION => return Err(StoreError::InvalidStoredSource),
                // Replacing an identical legacy payload only changes its storage wrapper. A
                // legacy payload with different evidence must never be allowed to take over the
                // same immutable source-version path.
                1 if stored.compiled == *compiled => {
                    replace_version(&versions_dir, &version_path, compiled)?;
                    SaveOutcome::NewVersion
                }
                1 => return Err(StoreError::InvalidStoredSource),
                version => return Err(StoreError::UnsupportedSchema(version)),
            },
            Err(StoreError::Io { error, .. }) if error.kind() == io::ErrorKind::NotFound => {
                write_new_version(&versions_dir, &version_path, compiled)?
            }
            Err(error) => return Err(error),
        };

        let language_key = pointer_key(Some(&compiled.evidence[0].transcript.language))?;
        write_pointer(&tracks_dir, &language_key, digest)?;
        if set_as_default {
            write_pointer(&tracks_dir, "default", digest)?;
        }
        Ok(outcome)
    }

    fn source_dir(&self, source: &CanonicalSource) -> PathBuf {
        let provider = match source.provider {
            SourceProvider::YouTube => "youtube",
        };
        self.root.join(provider).join(&source.source_id)
    }
}

fn verify_stored(
    compiled: &CompiledSource,
    expected_source: &CanonicalSource,
    language: Option<&str>,
    expected_digest: &str,
) -> Result<(), StoreError> {
    let stored_language = compiled
        .evidence
        .first()
        .map(|evidence| evidence.transcript.language.as_str());
    if compiled.source != *expected_source
        || version_digest(&compiled.source_version) != Some(expected_digest)
        || language.is_some_and(|language| stored_language != Some(language))
        || verify_compiled_source(compiled).is_err()
    {
        return Err(StoreError::InvalidStoredSource);
    }
    Ok(())
}

fn read_stored_source(path: &Path) -> Result<StoredSource, StoreError> {
    let bytes = read_limited(path, MAX_SOURCE_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|_| StoreError::InvalidStoredSource)
}

fn write_new_version(
    directory: &Path,
    destination: &Path,
    compiled: &CompiledSource,
) -> Result<SaveOutcome, StoreError> {
    let temporary = serialise_version(directory, compiled)?;
    match temporary.persist_noclobber(destination) {
        Ok(_) => Ok(SaveOutcome::NewVersion),
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {
            let winner = read_stored_source(destination)?;
            if winner.schema_version == STORE_SCHEMA_VERSION && winner.compiled == *compiled {
                Ok(SaveOutcome::ReusedVersion)
            } else {
                Err(StoreError::InvalidStoredSource)
            }
        }
        Err(_) => Err(StoreError::Persist("version")),
    }
}

fn replace_version(
    directory: &Path,
    destination: &Path,
    compiled: &CompiledSource,
) -> Result<(), StoreError> {
    serialise_version(directory, compiled)?
        .persist(destination)
        .map_err(|_| StoreError::Persist("version"))?;
    Ok(())
}

fn serialise_version(
    directory: &Path,
    compiled: &CompiledSource,
) -> Result<NamedTempFile, StoreError> {
    let mut temporary = NamedTempFile::new_in(directory).map_err(|error| StoreError::Io {
        stage: "creating cache version",
        error,
    })?;
    serde_json::to_writer_pretty(
        temporary.as_file_mut(),
        &StoredSource {
            schema_version: STORE_SCHEMA_VERSION,
            compiled: compiled.clone(),
        },
    )
    .map_err(StoreError::Serialise)?;
    temporary
        .as_file_mut()
        .write_all(b"\n")
        .map_err(|error| StoreError::Io {
            stage: "writing cache version",
            error,
        })?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| StoreError::Io {
            stage: "syncing cache version",
            error,
        })?;
    Ok(temporary)
}

fn write_pointer(directory: &Path, key: &str, digest: &str) -> Result<(), StoreError> {
    let destination = directory.join(format!("{key}.latest"));
    let mut temporary = NamedTempFile::new_in(directory).map_err(|error| StoreError::Io {
        stage: "creating cache pointer",
        error,
    })?;
    writeln!(temporary, "{digest}").map_err(|error| StoreError::Io {
        stage: "writing cache pointer",
        error,
    })?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| StoreError::Io {
            stage: "syncing cache pointer",
            error,
        })?;
    temporary
        .persist(destination)
        .map_err(|_| StoreError::Persist("pointer"))?;
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path).map_err(|error| StoreError::Io {
        stage: "creating cache directory",
        error,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
            StoreError::Io {
                stage: "securing cache directory",
                error,
            }
        })?;
    }
    Ok(())
}

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| StoreError::Io {
        stage: "reading cache metadata",
        error,
    })?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > limit
    {
        return Err(StoreError::InvalidStoredSource);
    }
    fs::read(path).map_err(|error| StoreError::Io {
        stage: "reading cache file",
        error,
    })
}

fn pointer_key(language: Option<&str>) -> Result<String, StoreError> {
    let value = language.unwrap_or("default");
    if value.is_empty()
        || value.len() > 64
        || language.is_some_and(|_| value == "default")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(StoreError::InvalidPointer);
    }
    Ok(value.to_owned())
}

fn version_digest(version: &str) -> Option<&str> {
    version
        .strip_prefix(VERSION_PREFIX)
        .filter(|value| is_digest(value))
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{Arc, Barrier},
        thread,
    };

    use serde_json::{Value, json};
    use tempfile::tempdir;

    use crate::fixture::compile_fixture;

    use super::{FileSourceStore, SaveOutcome, StoreError, version_digest};

    const FIXTURE: &str = "schema_version\t1\n\
source_url\thttps://youtu.be/dQw4w9WgXcQ\n\
title\tCached evidence\n\
creator\tOriel\n\
duration_ms\t10000\n\
language\ten\n\
caption_provenance\tmanual\n\
cue\t0\t10000\tImmutable timestamped evidence.\n";

    #[test]
    fn round_trips_and_reuses_immutable_versions() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = FileSourceStore::new(directory.path());
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");

        assert_eq!(
            store.save(&compiled, true).expect("source should save"),
            SaveOutcome::NewVersion
        );
        assert_eq!(
            store.save(&compiled, true).expect("source should reuse"),
            SaveOutcome::ReusedVersion
        );
        assert_eq!(
            store
                .load_latest(&compiled.source, Some("en"))
                .expect("source should load"),
            Some(compiled.clone())
        );
        assert_eq!(
            store
                .load_latest(&compiled.source, None)
                .expect("default source should load"),
            Some(compiled)
        );
    }

    #[test]
    fn a_missing_pointer_is_a_cache_miss() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = FileSourceStore::new(directory.path());
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        assert_eq!(
            store
                .load_latest(&compiled.source, Some("en"))
                .expect("cache miss should be valid"),
            None
        );
    }

    #[test]
    fn rejects_tampered_pointers() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = FileSourceStore::new(directory.path());
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        store.save(&compiled, true).expect("source should save");
        let pointer = directory
            .path()
            .join("youtube/dQw4w9WgXcQ/tracks/en.latest");
        fs::write(pointer, "../../unexpected\n").expect("pointer should be writable");

        assert!(matches!(
            store.load_latest(&compiled.source, Some("en")),
            Err(StoreError::InvalidPointer)
        ));
    }

    #[test]
    fn a_known_legacy_schema_is_reacquired_and_upgraded() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = FileSourceStore::new(directory.path());
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        store.save(&compiled, true).expect("source should save");
        let version = version_path(directory.path(), &compiled.source_version);
        set_schema_version(&version, 1);

        assert_eq!(
            store
                .load_latest(&compiled.source, Some("en"))
                .expect("legacy cache should request reacquisition"),
            None
        );
        assert_eq!(
            store
                .save(&compiled, true)
                .expect("reacquired source should upgrade the legacy entry"),
            SaveOutcome::NewVersion
        );
        assert_eq!(
            store
                .load_latest(&compiled.source, Some("en"))
                .expect("upgraded source should load"),
            Some(compiled)
        );
    }

    #[test]
    fn an_unknown_future_schema_remains_fail_closed() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = FileSourceStore::new(directory.path());
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        store.save(&compiled, true).expect("source should save");
        let version = version_path(directory.path(), &compiled.source_version);
        set_schema_version(&version, 3);

        assert!(matches!(
            store.load_latest(&compiled.source, Some("en")),
            Err(StoreError::UnsupportedSchema(3))
        ));
        assert!(matches!(
            store.save(&compiled, true),
            Err(StoreError::UnsupportedSchema(3))
        ));
    }

    #[test]
    fn concurrent_identical_writes_reuse_the_immutable_winner() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = Arc::new(FileSourceStore::new(directory.path()));
        let compiled = Arc::new(compile_fixture(FIXTURE).expect("fixture should compile"));
        let barrier = Arc::new(Barrier::new(3));
        let mut workers = Vec::new();

        for _ in 0..2 {
            let store = Arc::clone(&store);
            let compiled = Arc::clone(&compiled);
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                store.save(&compiled, true)
            }));
        }
        barrier.wait();

        let outcomes = workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("cache writer should not panic")
                    .expect("identical cache writer should succeed")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == SaveOutcome::NewVersion)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == SaveOutcome::ReusedVersion)
                .count(),
            1
        );
        assert_eq!(
            store
                .load_latest(&compiled.source, Some("en"))
                .expect("winning source should load"),
            Some((*compiled).clone())
        );
    }

    #[test]
    fn an_inconsistent_immutable_destination_is_rejected() {
        let directory = tempdir().expect("temporary cache should exist");
        let store = FileSourceStore::new(directory.path());
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        store.save(&compiled, true).expect("source should save");
        let mut inconsistent = compiled;
        inconsistent.acquisition.adapter = "different-adapter".to_owned();

        assert!(matches!(
            store.save(&inconsistent, true),
            Err(StoreError::InvalidStoredSource)
        ));
    }

    fn version_path(root: &Path, source_version: &str) -> PathBuf {
        let digest = version_digest(source_version).expect("fixture version should have a digest");
        root.join("youtube/dQw4w9WgXcQ/versions")
            .join(format!("{digest}.json"))
    }

    fn set_schema_version(path: &Path, schema_version: u32) {
        let mut stored: Value = serde_json::from_slice(
            &fs::read(path).expect("stored source should be readable for test setup"),
        )
        .expect("stored source should be valid JSON");
        stored["schema_version"] = json!(schema_version);
        fs::write(
            path,
            serde_json::to_vec_pretty(&stored).expect("stored source should serialise"),
        )
        .expect("stored source should be writable for test setup");
    }
}
