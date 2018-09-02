use std::io::{ Result };
use std::io::prelude::*;

use std::fs::File;
use std::io::BufReader;

use std::collections::hash_map::{ DefaultHasher, HashMap };
use std::hash::{ Hash, Hasher };

use tree::GenericTree;
use tree::range_tree::RangeTree;
use tree::segment_tree::SegmentTree;
use tree::heap::MinHeap;
use monoid::Monoid;

use itertools::Itertools;

use chrono::NaiveDateTime;

type Date = NaiveDateTime;
type QueryId = u64;
type DateId = usize;

#[derive(Clone)]
pub struct Solver {
    queries: HashMap<QueryId, String>,      // Storage of queries
    dates: HashMap<Date, DateId>,           // Storage of dates
    grouped_queries: Vec<Vec<QueryId>>,
    date_range_tree: RangeTree<Date>,       // Range tree of Date for finding correct ranges in log(N)
    segment_tree: SegmentTree<usize>        // Segment tree for finding number of queries in a range in log(N)
}

impl Solver {
    /// Build data structures to answer queries efficiently
    pub fn new(tsv_filename: &str) -> Result<Self> {
        const TSV_SEP: char = '\t';

        let file = File::open(tsv_filename)?;
        let reader = BufReader::new(file);
        
        // Lazy parsing of the TSV file into (Date, String) tuples
        let tsv_fields = reader.lines().map(|maybe_line| {
            let line = maybe_line.unwrap();
            let fields: Vec<&str> = line.split(TSV_SEP).collect();
            (Date::parse_from_str(fields[0], "%F %T").unwrap(), String::from(fields[1]))
        });

        // Hash queries and keep them in a hashmap
        // We also maintain a vector of (Date, QueryId) for later
        // We have to process N queries
        let mut queries: HashMap<QueryId, String> = HashMap::new();
        let mut entries: Vec<(Date, QueryId)> = Vec::new();

        for (date, query) in tsv_fields {
            let mut hasher = DefaultHasher::new();
            query.hash(&mut hasher);
            let query_hash = hasher.finish();

            queries.entry(query_hash).or_insert(query);
            entries.push((date, query_hash));
        }

        // Sort entries, group and index them by date
        // Sorting: O(N log N)
        // Grouping: O(N)
        entries.sort();
        let mut grouped_queries: Vec<(Date, Vec<QueryId>)> = Vec::new();
        for (date, query_group) in &entries.into_iter().group_by(|&entry| entry.0) {
            let queries: Vec<QueryId> = query_group.into_iter().map(|(_, query)| query).collect();
            grouped_queries.push((date, queries));
        }

        // Build the mapping of Date -> DateId
        let mut date_map = HashMap::with_capacity(grouped_queries.len());
        for (date_id, &(date, _)) in grouped_queries.iter().enumerate() {
            date_map.insert(date.clone(), date_id);
        }

        // Collect leaves of the range tree of dates
        let range_tree_leaves: Vec<Date> = grouped_queries.iter().map(|&(date, _)| date).collect();

        // Collect leaves of the segment tree of number of queries
        let seg_tree_leaves: Vec<usize> = grouped_queries.iter().map(|&(_, ref v)| v.len()).collect();

        Ok(Solver {
            queries: queries,
            dates: date_map,
            grouped_queries: grouped_queries.iter().map(|&(_, ref v)| v.clone()).collect(),
            date_range_tree: RangeTree::with_leaves(&range_tree_leaves),
            segment_tree: SegmentTree::with_leaves(&seg_tree_leaves)
        })
    }

    fn find_date_range_ids(&self, from: &Date, to: &Date) -> Option<(DateId, DateId)> {
        let maybe_range = self.date_range_tree.largest_range(from, to);
        match maybe_range {
            None => None,

            Some((from, to)) => {
                self.dates.get(&from).and_then(|from_id| {
                    self.dates.get(&to).and_then(|to_id| {
                        Some((*from_id, *to_id))
                    })
                })
            }
        }
    }

    /// Query number of queries in a range
    pub fn query_count(&self, from: &Date, to: &Date) -> usize {
        match self.find_date_range_ids(from, to) {
            Some((from_id, to_id)) => {
                self.segment_tree.query(from_id, to_id)
            },

            _ => 0
        }
    }

    /// Query k most frequent requests in a range
    pub fn query_k_count(&self, from: &Date, to: &Date, k: usize) -> Vec<(String, usize)> {
        match self.find_date_range_ids(from, to) {
            Some((from_id, to_id)) if k > 0 => {
                let mut query_counts: HashMap<QueryId, usize> = HashMap::new();

                // Count queries in the given range
                for date_id in from_id .. to_id + 1 {
                    for query_id in self.grouped_queries[date_id].iter() {
                        let count = query_counts.entry(*query_id)
                                                .or_insert(0);
                        *count += 1;
                    }
                }

                // To solve the problem we maintain a min-heap with at most the k most frequent queries
                let mut solution = MinHeap::new();

                // 1. fill the min-heap with the first k elements.
                let query_count_iter_k = query_counts.iter().take(k).map(|(query, count)| (count, query));
                for query_count in query_count_iter_k {
                    solution.insert(query_count);
                }

                // 2. for the remaining elements just look if their count is greater than the one
                //    of the root of the min heap. If so, remove replace the root with this
                //    element.
                let query_count_iter_others = query_counts.iter().skip(k).map(|(query, count)| (count, query));
                for query_count in query_count_iter_others {
                    let head = solution.peek().unwrap().clone();
                    if head < query_count {
                        solution.extract();
                        solution.insert(query_count);
                    }
                }

                // Transfor the heap into a Vec
                solution.into_iter()
                        .map(|(count, query_id)| (self.queries.get(query_id).unwrap().clone(), count.clone()))
                        .collect()
            },

            _ => Vec::new()
        }
    }
}

impl Monoid for usize {
    fn m_empty() -> Self {
        0
    }

    fn m_append(&self, other: &Self) -> Self {
        self + other
    }
}
