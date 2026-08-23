//! Pure filtering, sorting, and grouping helpers over `[Task]`.
//!
//! These take plain data (`&Task` / `&[Task]` + `Filter`/`Sort` + flags) and
//! return decisions or orderings. They hold no view state, so both the TUI's
//! `recompute_visible` (which owns the visible-index cache) and the CLI's
//! `list`/`listpri`/`listproj`/`listcon` commands reuse them.

use std::cmp::Ordering;

use chrono::{Datelike, Days, NaiveDate};

use crate::app::{Filter, Sort, WeekStart};
use crate::search::subseq_match_ci;
use crate::threshold;
use crate::todo::{self, Task};

/// Which canonical bucket a List-view row belongs to when the active sort is
/// `Sort::Due`. `NoDue` covers tasks with no `due:` tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListDueBucket {
    Overdue,
    Today,
    ThisWeek,
    NextWeek,
    Later,
    NoDue,
}

impl ListDueBucket {
    pub fn label(self) -> &'static str {
        match self {
            ListDueBucket::Overdue => "OVERDUE",
            ListDueBucket::Today => "TODAY",
            ListDueBucket::ThisWeek => "THIS WEEK",
            ListDueBucket::NextWeek => "NEXT WEEK",
            ListDueBucket::Later => "LATER",
            ListDueBucket::NoDue => "NO DUE DATE",
        }
    }
}

pub fn get_week_cutoff(today: &str, week_start: &WeekStart) -> Option<(String, String)> {
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").ok()?;
    let weekday = today.weekday();

    let days_from_start_week = match week_start {
        WeekStart::Sunday => weekday.num_days_from_sunday(),
        WeekStart::Monday => weekday.num_days_from_monday(),
    };

    let days_til_week_end = 6 - days_from_start_week;

    let end_this_week = today.checked_add_days(Days::new(days_til_week_end as u64))?;
    let end_next_week = today.checked_add_days(Days::new((days_til_week_end + 7) as u64))?;

    Some((end_this_week.to_string(), end_next_week.to_string()))
}

/// If the date cannot be parsed we assign to Later
pub fn due_bucket(task: &Task, today: &str, week_start: &WeekStart) -> ListDueBucket {
    match task.due.as_deref() {
        None => ListDueBucket::NoDue,
        Some(d) => {
            let Some((this_week, next_week)) = get_week_cutoff(today, week_start) else {
                return ListDueBucket::Later;
            };

            match d.cmp(today) {
                Ordering::Less => ListDueBucket::Overdue,
                Ordering::Equal => ListDueBucket::Today,
                Ordering::Greater if d <= this_week.as_str() => ListDueBucket::ThisWeek,
                Ordering::Greater if d <= next_week.as_str() => ListDueBucket::NextWeek,
                Ordering::Greater => ListDueBucket::Later,
            }
        }
    }
}

pub fn sort_by_prefs(idxs: &mut [usize], tasks: &[Task], sort: Sort) {
    match sort {
        Sort::Priority => sort_families(idxs, tasks, cmp_priority(tasks)),
        Sort::Due => sort_families(idxs, tasks, cmp_due(tasks)),
        Sort::File => { /* preserve order */ }
        // Project grouping is an *expansion*, not a permutation: a task with
        // several projects must appear once per project. Handled by
        // `expand_by_project`, which returns a new (longer) list.
        Sort::Project => {}
    }
}

/// Expand a visible index list into project groups. Each family is placed
/// under every project its *root* carries, in alphabetical project order,
/// with a trailing NO PROJECT group for families whose root has none. The
/// returned `(idxs, groups)` are parallel: `groups[i]` is the project bucket
/// of `idxs[i]`. Multi-project tasks repeat across groups (per the sort's
/// contract), and families always stay contiguous.
pub fn expand_by_project(
    idxs: &[usize],
    tasks: &[Task],
) -> (Vec<usize>, Vec<crate::app::GroupKey>) {
    use std::collections::BTreeMap;
    let visible: std::collections::HashSet<usize> = idxs.iter().copied().collect();

    // Family root -> members in file order.
    let mut families: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut root_of: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &i in idxs.iter() {
        let root = *root_of
            .entry(i)
            .or_insert_with(|| family_root(tasks, i, &visible));
        families.entry(root).or_default().push(i);
    }

    // Root -> sorted unique project names.
    let mut root_projects: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut all_projects: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for &root in families.keys() {
        let mut ps: Vec<String> = tasks[root].projects.clone();
        ps.sort();
        ps.dedup();
        for p in &ps {
            all_projects.insert(p.clone());
        }
        root_projects.insert(root, ps);
    }

    let mut out: Vec<usize> = Vec::with_capacity(idxs.len() + all_projects.len());
    let mut groups: Vec<crate::app::GroupKey> = Vec::with_capacity(out.capacity());

    for proj in all_projects {
        for (&root, members) in families.iter() {
            if root_projects[&root].iter().any(|p| p == &proj) {
                for &m in members {
                    out.push(m);
                    groups.push(crate::app::GroupKey::ListProject(Some(proj.clone())));
                }
            }
        }
    }

    // NO PROJECT group: families whose root carries no project tag.
    for (&root, members) in families.iter() {
        if root_projects[&root].is_empty() {
            for &m in members {
                out.push(m);
                groups.push(crate::app::GroupKey::ListProject(None));
            }
        }
    }

    (out, groups)
}

/// Topmost visible ancestor of `idx` (itself when no visible ancestor exists
/// above it). A subtask is grouped with the nearest ancestor that also passes
/// the active filter, so children never float away from their parent in a
/// sorted view.
pub fn family_root(
    tasks: &[Task],
    idx: usize,
    visible: &std::collections::HashSet<usize>,
) -> usize {
    let mut r = idx;
    while let Some(p) = crate::todo::parent_index(tasks, r) {
        if visible.contains(&p) {
            r = p;
        } else {
            break;
        }
    }
    r
}

/// Sort `idxs` so each family (a root plus its descendants, in file order)
/// stays contiguous, ordered by the family root's sort key. Children keep
/// their file order within the family.
fn sort_families<F>(idxs: &mut [usize], tasks: &[Task], cmp: F)
where
    F: Fn(&usize, &usize) -> Ordering,
{
    use std::collections::HashMap;
    let visible: std::collections::HashSet<usize> = idxs.iter().copied().collect();
    let mut families: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut root_order: Vec<usize> = Vec::new();
    let mut root_of: HashMap<usize, usize> = HashMap::new();

    for &i in idxs.iter() {
        let root = *root_of
            .entry(i)
            .or_insert_with(|| family_root(tasks, i, &visible));
        families
            .entry(root)
            .or_insert_with(|| {
                root_order.push(root);
                Vec::new()
            })
            .push(i);
    }

    root_order.sort_by(|a, b| cmp(a, b));

    let mut out = Vec::with_capacity(idxs.len());
    for root in root_order {
        out.extend(families[&root].iter().copied());
    }
    idxs.copy_from_slice(&out);
}

/// Project / context / search predicate, shared by every view that honors
/// user filters. `needle` matches as a case-insensitive subsequence of the
/// task body — chars must appear in order, gaps allowed.
pub fn passes_user_filter(t: &Task, filter: &Filter, needle: Option<&str>) -> bool {
    if let Some(p) = &filter.project
        && !t.projects.iter().any(|x| x == p)
    {
        return false;
    }
    if let Some(c) = &filter.context
        && !t.contexts.iter().any(|x| x == c)
    {
        return false;
    }
    if let Some(needle) = needle {
        let body = todo::body_after_priority(&t.raw);
        if subseq_match_ci(body, needle).is_none() {
            return false;
        }
    }
    true
}

pub fn list_predicate(
    t: &Task,
    show_done: bool,
    show_future: bool,
    today: &str,
    filter: &Filter,
    needle: Option<&str>,
) -> bool {
    if t.done && !show_done {
        return false;
    }
    if !show_future && is_future_threshold(t, today) {
        return false;
    }
    passes_user_filter(t, filter, needle)
}

/// Family-atomic list predicate: whether task `i` should show under the active
/// filter. The *family root* (topmost ancestor) decides the user filter
/// (project / context / search) for the whole family, so a subtask appears
/// alongside its root even when it carries no matching tags itself — and a
/// matching subtask never surfaces without its root (option A). The structural
/// filters (`done`, future threshold) still apply per task, so a completed
/// subtask stays hidden even when its root is active.
pub fn list_predicate_family(
    tasks: &[Task],
    i: usize,
    show_done: bool,
    show_future: bool,
    today: &str,
    filter: &Filter,
    needle: Option<&str>,
) -> bool {
    let t = &tasks[i];
    if t.done && !show_done {
        return false;
    }
    if !show_future && is_future_threshold(t, today) {
        return false;
    }
    let root = crate::todo::abs_family_root(tasks, i);
    passes_user_filter(&tasks[root], filter, needle)
}

/// True when the task carries a `t:` value that resolves to a date strictly
/// after `today`. Malformed values, missing anchors for relative offsets,
/// and arithmetic overflow all leave the task visible — better to surface a
/// task the user might miss than to hide it because of a bad threshold.
pub fn is_future_threshold(t: &Task, today: &str) -> bool {
    let Some(raw) = t.threshold.as_deref() else {
        return false;
    };
    let Some(spec) = threshold::parse_threshold(raw) else {
        return false;
    };
    let Some(date) = threshold::resolve(&spec, t.due.as_deref(), t.created_date.as_deref()) else {
        return false;
    };
    date.format("%Y-%m-%d").to_string().as_str() > today
}

/// Sort by priority asc (None last), tie-broken by due-date asc.
fn cmp_priority(tasks: &[Task]) -> impl Fn(&usize, &usize) -> Ordering + '_ {
    |&a, &b| {
        let ta = &tasks[a];
        let tb = &tasks[b];
        let pa = ta.priority.unwrap_or('Z');
        let pb = tb.priority.unwrap_or('Z');
        pa.cmp(&pb).then_with(|| {
            ta.due
                .as_deref()
                .unwrap_or("z")
                .cmp(tb.due.as_deref().unwrap_or("z"))
        })
    }
}

/// Sort by due-date asc (None last).
fn cmp_due(tasks: &[Task]) -> impl Fn(&usize, &usize) -> Ordering + '_ {
    |&a, &b| {
        tasks[a]
            .due
            .as_deref()
            .unwrap_or("z")
            .cmp(tasks[b].due.as_deref().unwrap_or("z"))
    }
}

/// Order projects/contexts the same way the filter sidebar does:
/// count descending, then name ascending. Used by both the picker and
/// the sidebar so j/k advances visibly down the list.
pub fn ordered_unique<F>(tasks: &[Task], pick: F) -> Vec<(String, usize)>
where
    F: Fn(&Task) -> &Vec<String>,
{
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for t in tasks.iter().filter(|t| !t.done) {
        for v in pick(t) {
            *counts.entry(v.clone()).or_insert(0) += 1;
        }
    }
    let mut out: Vec<(String, usize)> = counts.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

pub fn unique_values<F>(tasks: &[Task], pick: F) -> Vec<String>
where
    F: Fn(&Task) -> &Vec<String>,
{
    ordered_unique(tasks, pick)
        .into_iter()
        .map(|(n, _)| n)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_values_dedups_and_sorts() {
        let raw = "(A) 2026-05-01 a +work +health\n2026-05-01 b +work\n2026-05-01 c +health\n";
        let tasks = crate::todo::parse_file(raw);
        let projects = unique_values(&tasks, |t| &t.projects);
        assert_eq!(projects, vec!["health".to_string(), "work".to_string()]);
    }

    #[test]
    fn get_week_cutoffs_for_all_configs() {
        let (end_this_week, end_next_week) = get_week_cutoff("2026-06-18", &WeekStart::Sunday)
            .expect("unable to get the week cutoff date");
        assert_eq!(end_this_week, "2026-06-20");
        assert_eq!(end_next_week, "2026-06-27");

        let (end_this_week, end_next_week) = get_week_cutoff("2026-06-18", &WeekStart::Monday)
            .expect("unable to get the week cutoff date");
        assert_eq!(end_this_week, "2026-06-21");
        assert_eq!(end_next_week, "2026-06-28");
    }

    #[test]
    fn sort_by_prefs_keeps_families_together() {
        // Parent (no priority) + child (priority A); sibling (priority A).
        // Under priority sort the sibling family (A) comes first, then the
        // parent family, with the child pinned right after its parent even
        // though the child itself is also priority A.
        let tasks = crate::todo::parse_file("parent\n  (A) child\n(A) sibling\n");
        let mut idxs = vec![0, 1, 2];
        sort_by_prefs(&mut idxs, &tasks, Sort::Priority);
        assert_eq!(idxs, vec![2, 0, 1]);
    }

    #[test]
    fn family_root_skips_filtered_out_ancestors() {
        let tasks = crate::todo::parse_file("parent\n  child\n");
        let visible: std::collections::HashSet<usize> = [1].iter().copied().collect();
        // Parent isn't visible, so the child roots itself.
        assert_eq!(family_root(&tasks, 1, &visible), 1);
        let visible: std::collections::HashSet<usize> = [0, 1].iter().copied().collect();
        assert_eq!(family_root(&tasks, 1, &visible), 0);
    }

    #[test]
    fn list_predicate_family_shows_subtasks_when_root_matches() {
        // Root carries @phone; its subtasks don't. All must show.
        let tasks = crate::todo::parse_file("Call dentist @phone\n  reschedule\n  pay\n");
        let filter = Filter {
            context: Some("phone".into()),
            ..Filter::default()
        };
        assert!(list_predicate_family(
            &tasks,
            0,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
        assert!(list_predicate_family(
            &tasks,
            1,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
        assert!(list_predicate_family(
            &tasks,
            2,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
    }

    #[test]
    fn list_predicate_family_root_decides_alone() {
        // Option A: only the root activates the family. A subtask carrying
        // @phone under an untagged root must NOT surface without its root.
        let tasks = crate::todo::parse_file("root\n  child @phone\n");
        let filter = Filter {
            context: Some("phone".into()),
            ..Filter::default()
        };
        assert!(!list_predicate_family(
            &tasks,
            0,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
        assert!(!list_predicate_family(
            &tasks,
            1,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
    }

    #[test]
    fn list_predicate_family_applies_search_to_root() {
        // Search matches on the root's body decide the whole family.
        let tasks = crate::todo::parse_file("plan party\n  book venue\n");
        let filter = Filter::default();
        let needle = Some("party");
        assert!(list_predicate_family(
            &tasks,
            0,
            false,
            false,
            "2026-05-06",
            &filter,
            needle
        ));
        assert!(list_predicate_family(
            &tasks,
            1,
            false,
            false,
            "2026-05-06",
            &filter,
            needle
        ));
        // Search term only inside a child does not surface the family.
        let tasks2 = crate::todo::parse_file("plan\n  book party\n");
        assert!(!list_predicate_family(
            &tasks2,
            0,
            false,
            false,
            "2026-05-06",
            &filter,
            needle
        ));
        assert!(!list_predicate_family(
            &tasks2,
            1,
            false,
            false,
            "2026-05-06",
            &filter,
            needle
        ));
    }

    #[test]
    fn list_predicate_family_hides_done_subtask() {
        // Structural filters still apply per task: a done subtask stays
        // hidden even when its root is active.
        let tasks = crate::todo::parse_file("root @phone\n  x 2026-05-06 child\n");
        let filter = Filter {
            context: Some("phone".into()),
            ..Filter::default()
        };
        assert!(list_predicate_family(
            &tasks,
            0,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
        assert!(!list_predicate_family(
            &tasks,
            1,
            false,
            false,
            "2026-05-06",
            &filter,
            None
        ));
        // With show_done, the done subtask shows again.
        assert!(list_predicate_family(
            &tasks,
            1,
            true,
            false,
            "2026-05-06",
            &filter,
            None
        ));
    }

    #[test]
    fn abs_family_root_resolves_topmost_ancestor() {
        let tasks = crate::todo::parse_file("a\n  b\n    c\n  d\n");
        assert_eq!(crate::todo::abs_family_root(&tasks, 2), 0);
        assert_eq!(crate::todo::abs_family_root(&tasks, 1), 0);
        assert_eq!(crate::todo::abs_family_root(&tasks, 0), 0);
        assert_eq!(crate::todo::abs_family_root(&tasks, 3), 0);
    }

    #[test]
    fn expand_by_project_groups_alphabetically_and_repeats_multi_project() {
        // Task 1 has +banana +apple (multi-project → appears under both);
        // task 0 has +apple only; task 2 has no project → NO PROJECT group.
        let tasks = crate::todo::parse_file("a +apple\nb +banana +apple\nc\n");
        let idxs = vec![0, 1, 2];
        let (expanded, groups) = expand_by_project(&idxs, &tasks);
        use crate::app::GroupKey;
        assert_eq!(expanded, vec![0, 1, 1, 2]);
        assert_eq!(groups[0], GroupKey::ListProject(Some("apple".into())));
        assert_eq!(groups[1], GroupKey::ListProject(Some("apple".into())));
        assert_eq!(groups[2], GroupKey::ListProject(Some("banana".into())));
        assert_eq!(groups[3], GroupKey::ListProject(None));
    }

    #[test]
    fn expand_by_project_keeps_family_together_under_root_projects() {
        // Parent +apple with a child (no own project). The family must stay
        // contiguous under +apple, grouped by the root's project.
        let tasks = crate::todo::parse_file("parent +apple\n  child\n");
        let idxs = vec![0, 1];
        let (expanded, groups) = expand_by_project(&idxs, &tasks);
        use crate::app::GroupKey;
        assert_eq!(expanded, vec![0, 1]);
        assert_eq!(
            groups,
            vec![
                GroupKey::ListProject(Some("apple".into())),
                GroupKey::ListProject(Some("apple".into())),
            ]
        );
    }

    #[test]
    fn expand_by_project_all_untagged_lands_in_no_project() {
        let tasks = crate::todo::parse_file("a\nb\n");
        let (expanded, groups) = expand_by_project(&[0, 1], &tasks);
        use crate::app::GroupKey;
        assert_eq!(expanded, vec![0, 1]);
        assert_eq!(
            groups,
            vec![GroupKey::ListProject(None), GroupKey::ListProject(None)]
        );
    }
}
