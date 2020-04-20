file:///home/patrick/rust/wolfram01/README.md {"mtime":1587337471550,"ctime":1587337209206,"size":112,"etag":"350h06i413j","orphaned":false}
# wolfram01

**Creating 3D graphs from simple edge substitution rules.**

see: https://writings.stephenwolfram.com/2020/04/finally-we-may-have-a-path-to-the-fundamental-theory-of-physics-and-its-beautiful/

Here's a description of what the program does:

0. You will need to read Stephen Wolfram's article to some extent to get a feel
for what's going on.

1. It reads four command line arguments: a) an edge pattern to find b) an edge
pattern to substitute for the one that's been found c) a starting graph of edges
d) the number of generations to run. If you leave any spaces in any of these arguments
then you have to put them in quotes::

    [[1,2],[1,3]] [[1,2],[1,5]]
    or
    '[[1, 2], [1, 3]]' '[[1, 2], [1, 5]]' etc

2. Edge patterns are described using numbers 0 to 20 to allow them to be used as
index values to a normal Vec (rather than a HashMap which would have been 
slower). Each number represents a node, where different numbers can represent the same
node but the numbering must be consistent. i.e. [[1,2],[1,3]] will look for an edge connecting
two nodes or a node connected to itself representing [1,2]. It will then search for
another edge connecting the same node matching 1 (and, possibly, 2) in the first
edge to a different node, or possibly the same one. So if the graph contains an edge
[101,101] that can match the [1,2]. The algorithm will then look for another edge somewhere in the graph [101,x] where x can be anything including 101. This is my understanding of process from reading the blog and associated descriptive pages.

3. The second part of the substitution description gives a list of edges that either
re-use nodes referenced in the search part or create new nodes if they use a number
not used in the search part of the rule description. i.e. using the example if the
second part of the rule was [[1,2],[1,4],[2,4],[3,4]] and it had found [101,101] and
[101,102] as a match it would replace those edges with [101,101], [101,103], [101,103]
and [102,103]

4. Referring to the code: the from and to parts of the rule are loaded as well as
the starting graph. The graph is then repeatedly searched to find a set of matching
edges which are removed and put into another list. When no more matching edges can
be found the other list is traversed and each matching set of edges converted into
a new set. This process is repeated for the number of generations specified.

5. The matching part of the algorithm uses a recursive backtracking function called
recurse() which in turn uses a checking function called match_flat() to verify if the edges
being checked are compatible with the 'from' pattern. Both lists are 'flattened'
to make the process quicker. The function works by making a lookup table to match
each of the nodes, returning early if there is a conflict.

6. After the number of generations the nodes are spread over block of space then
'annealed' into a stable form balancing a repulsive force from the centre of gravity
of all the node (inverse distance) with an attractive force along each edge (proportional
with distance but with a switch to repulsion when too near.

7. The nodes are then used to form OpenGL vertices with line segments along each
edge and drawn using rust_pi3d

