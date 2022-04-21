use crate::{MatchGroup, PatternList};
use bstr::{BStr, ByteSlice};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A marker trait to identify the type of a description.
pub trait Tag: Clone + PartialEq + Eq + std::fmt::Debug + std::hash::Hash + Ord + PartialOrd {
    /// The value associated with a pattern.
    type Value: PartialEq + Eq + std::fmt::Debug + std::hash::Hash + Ord + PartialOrd + Clone;
}

/// Identify ignore patterns.
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone)]
pub struct Ignore;

impl Tag for Ignore {
    type Value = ();
}

/// Identify patterns with attributes.
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone)]
pub struct Attributes;

/// Describes a matching value within a [`MatchGroup`].
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone)]
pub struct Match<'a, T> {
    pub pattern: &'a git_glob::Pattern,
    /// The value associated with the pattern.
    pub value: &'a T,
    /// The path to the source from which the pattern was loaded, or `None` if it was specified by other means.
    pub source: Option<&'a Path>,
    /// The line at which the pattern was found in its `source` file, or the occurrence in which it was provided.
    pub sequence_number: usize,
}

impl Tag for Attributes {
    /// TODO: identify the actual value, should be name/State pairs, but there is the question of storage.
    type Value = ();
}

impl<T> MatchGroup<T>
where
    T: Tag,
{
    /// Match `relative_path`, a path relative to the repository containing all patterns,
    pub fn pattern_matching_relative_path<'a>(
        &self,
        relative_path: impl Into<&'a BStr>,
        is_dir: bool,
        case: git_glob::pattern::Case,
    ) -> Option<Match<'_, T::Value>> {
        let relative_path = relative_path.into();
        let basename_pos = relative_path.rfind(b"/").map(|p| p + 1);
        self.patterns
            .iter()
            .rev()
            .find_map(|pl| pl.pattern_matching_relative_path(relative_path, basename_pos, is_dir, case))
    }
}

impl MatchGroup<Ignore> {
    /// See [PatternList::<Ignore>::from_overrides()] for details.
    pub fn from_overrides(patterns: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        MatchGroup {
            patterns: vec![PatternList::<Ignore>::from_overrides(patterns)],
        }
    }
}

impl<T> PatternList<T>
where
    T: Tag,
{
    fn pattern_matching_relative_path(
        &self,
        relative_path: &BStr,
        basename_pos: Option<usize>,
        is_dir: bool,
        case: git_glob::pattern::Case,
    ) -> Option<Match<'_, T::Value>> {
        let (relative_path, basename_start_pos) = self
            .base
            .as_deref()
            .map(|base| {
                (
                    relative_path
                        .strip_prefix(base.as_slice())
                        .expect("input paths must be relative to base")
                        .as_bstr(),
                    basename_pos.map(|pos| pos - base.len()),
                )
            })
            .unwrap_or((relative_path, basename_pos));
        self.patterns.iter().rev().find_map(|(pattern, value, seq_id)| {
            pattern
                .matches_repo_relative_path(relative_path, basename_start_pos, is_dir, case)
                .then(|| Match {
                    pattern,
                    value,
                    source: self.source.as_deref(),
                    sequence_number: *seq_id,
                })
        })
    }
}

impl PatternList<Ignore> {
    /// Parse a list of patterns, using slashes as path separators
    pub fn from_overrides(patterns: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        PatternList {
            patterns: patterns
                .into_iter()
                .map(Into::into)
                .enumerate()
                .filter_map(|(seq_id, pattern)| {
                    let pattern = git_features::path::into_bytes(PathBuf::from(pattern)).ok()?;
                    git_glob::parse(pattern.as_ref()).map(|p| (p, (), seq_id))
                })
                .collect(),
            source: None,
            base: None,
        }
    }
}