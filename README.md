# Algolia technical test

Subject: <https://gist.github.com/sfriquet/55b18848d6d58b8185bbada81c620c4a>

## Requirements

- Cargo
- rustc

## Instructions

```bash
git clone https://github.com/lambdax-x/algolia-challenge
cd algolia-challenge
cargo run --release
```

Go to <http://127.0.0.1:8000> and follow the instructions.

## How does it work?

### Counting globally

Counting the number of queries in a time range is optimized using mainly two data structures: a range tree and a segment
tree. Given a time range, the range tree allows finding the largest valid range included in the one given in O(log N)
operations and the tree requires O(N) storage (it is balanced and has N leafs). After a valid range has been found (left
and right bounds) these bounds can be used to query a segment tree and compute the number of queries in this range. This
is also done in O(log N) operations, the segment tree also requires O(N) storage.

### Distinct count

Counting the number of distinct queries in a time range also uses the range tree to find the valid time range
corresponding to the input. Each query identifier is added to the hash set. This requires O(N) space and O(N)
operations. There are several papers on sub-linear distinct counting in large dataset, among them:

- LogLog, Super-LogLog, *Marianne Durand and Philippe Flajolet*: http://algo.inria.fr/flajolet/Publications/DuFl03-LNCS.pdf
- HyperLogLog
- MinCount, *Frédéric Giroire*: http://www-sop.inria.fr/members/Frederic.Giroire/publis/Gi05.pdf
- Count-Min Sketch, *Graham Cormode*: http://dimacs.rutgers.edu/~graham/pubs/papers/cmencyc.pdf

### Counting popular queries

Again, the range tree is used to find a valid time range. Counting queries in this range is done in a hash map, this
requires O(N) operations and O(N) storage. Selecting the K most frequent queries is then done using a min-heap in which
queries are inserted if their count is greater than the root of the heap, in which case the root is removed in order to
keep at most K queries in it. This requires again O(N log K) operations but O(K) storage.
