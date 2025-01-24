= Endgames =
1. Need a way for threads to analyse each endgame
2. Need a way for the result of each analysis to be stored for later use

One way to implement requirement 1 is to have an atomic which starts at 0 and goes to the maximum number of positions.
Each thread would read the atomic and increment it by 1 (atomic read and add).
The thread would then turn this into a position and analyse it.
Each position must be analysed. To check this, start with a database of 0s, have all analysis come back as 1, read back every position.

One way to implement requirement 2 is to have each position map to an index which can then be used as an index into an array.
We can imagine the position as a list of (12) numbers which are the path down a (12-deep) tree.
Each number in this list is the number of stones in the corresponding slot.
So if the position is:
 0 0 0 0 0 0
0           0
 1 0 0 0 0 0
Then the list of numbers is [1, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0]
which represents taking path 1 at the first node and path 0 for the other nodes.

In order to turn this list of numbers into an index we need to know how many leaves we have skipped.
As a simple example, take a smaller list with just 3 slots and 2 stones, eg:
[0, 1, 1]
The tree would look like
            |
 ------------------------
 |              |       |
 0------------  1-----  2
 |      |    |  |    |  |
 0----  1--  2  0--  1  0
 | | |  | |  |  | |  |  |
 0 1 2  0 1  0  0 1  0  0

 0 1 2  3 4  5  6 7  8  9

Or reversed (the maths works out simpler later):
      |
 -----------
 |  |      |
 2  1--    0-------
 |  | |    | |    |
 0  1 0--  2 1--  0----
 |  | | |  | | |  | | |
 0  0 1 0  0 1 0  2 1 0

 0  1 2 3  4 5 6  7 8 9
           0 1 2  3 4 5

Using the second tree, if we have [0, 1, 1]:
We first need to skip all the nodes from 2 and 1.
Given a depth of d and a number of stones n, the number of nodes skipped is:
(d+n)!/((n-1)!(d+1)!)
which is a binomial of (d+n, n-1) [proof below]
so the algorithm is:
for x in list:
    d -= 1
    n -= x
    sum += binomial(d+n, n-1)
The only exception is that this doesn't handle n=0 well so just add an if n=0 return sum

Proof: (above text is wrong)
Each tree is defined by a depth (d) and a number of stones (n). The above tree has d=3, n=2.
Each branch of the tree is its own tree, minus a root node for example the 2-0-0 branch:
 2 is the root node, we are left with 0-0, which is a tree with n=0, d=2

The number of ways to arrange n objects across d bins is a stars and bars problem and is:
 binomial(n + d - 1, d - 1) == binomial(n + d - 1, n)
https://en.wikipedia.org/wiki/Stars_and_bars_(combinatorics)#Theorem_two_proof
We want n or fewer objects and the easiest way to think about this is by adding an extra bin to put our leftover objects:
 binomial(n + d, d) == binomial(n + d, n)

We then need to sum over each branch we skip. If we are skipping m branches:
 sum from n=0 to n=m [ binomial(n + d, n) ] = binomial(m + d + 1, m) == binomial(m + d + 1, d + 1)
https://www.wolframalpha.com/input?i=%28sum+n%3D0+to+n%3Dm+%28binom%28n%2Bd%2Cd%29%29%29
The value read from the slot (i) will be one more than m, therefore:
index = binomial(i + d, i - 1) == binomial(i + d, d + 1)


The issue is this is fast going from position to index but slow and difficult to go from index to position so isn't an ideal solution to requirement 1
Requirement 1 doesn't require being fast from position to index though so this doesn't matter


Ideally, when analysing endgames we would start with all positions with 2 stones, then 3 stones etc, so that each calculation can use the results of the previous.

      |
 ---------
 |  |    |
 2  1--  0----
 |  | |  | | |
 0  1 0  2 1 0
 |  | |  | | |
 0  0 1  0 1 2

 0  1 2  3 4 5


      |
 ------------------------
 |  |      |            |
 3  2----  1------      0------------
 |  | | |  | |   |      | |   |     |
 0  1 0 0  2 1-- 0----  3 2-- 1---- 0------
 |  | | |  | | | | | |  | | | | | | | | | |
 0  0 1 0  0 1 0 2 1 0  0 1 0 2 1 0 3 2 1 0
 |  | | |  | | | | | |  | | | | | | | | | |
 0  0 0 1  0 0 1 0 1 2  0 0 1 0 1 2 0 1 2 3

 0  1 2 3  4 5 6 7 8 9  0 1 2 3 4 5 6 7 8 9

Using the second tree, if we have [1, 0, 2, 0]:
We first need to skip all the nodes from 3 and 2.
Given a depth of d and a number of stones n, the number of nodes skipped is:
(d+n)!/((n-1)!(d+1)!)
which is a binomial of (d+n, n-1)
so the algorithm is:
for x in list:
    d -= 1
    n -= x
    sum += binomial(d+n, n-1)
The only exception is that this doesn't handle n=0 well so just add an if n=0 return sum

Proof:
Each tree is defined by a depth (d) and a number of stones (n). The above tree has d=3, n=2.
Each branch of the tree is its own tree, minus a root node.

The number of ways to arrange n objects across d bins is a stars and bars problem and is:
 binomial(n + d - 1, d - 1) == binomial(n + d - 1, n)

We then need to sum over each branch we skip. If we are skipping m branches:
 sum from n=0 to n=m [ binomial(n + d - 1, n) ] = binomial(m + d, m) == binomial(m + d, d)
The value read from the slot (i) will be one more than m, therefore:
index = binomial(i + d - 1, i - 1) == binomial(i + d - 1, d)


An alternative would be to have the endgame database be atomically written/read to and just have an "unsearched" tag/enum. (could still do this anyway)
A job passed to a thread could be a batch of positions to search.


Alternative idea:
Order the database such that all the entries with n stones are together, starting with 2 stones going up to max stones
Need a function which counts how many entries of n stones there are and one which maths it out.
Ideally would have a function which counts how many entries of n or fewer there are.
Need a function which takes a position and gets an index from it, both by counting and by mathsing it

Need a function which takes a positions and "prints" every position after it in a given batch

for now, just going to lock the database behind a mutex and hope that doesn't slow things down too much








Dear future me,
position_from_index() and index_from_position() are now working and are proved by check_indexing()
They are implemented for having a table per number of stones, not for the "full" table (see above)
calculate_endgames() and functions that go with it I think still use the old method and need updating 
The idea is to have a vector of references to arrays, where each array contains the endgame analyses for a single stone count
Then slowly increase the "max number of stones" using the previous tables.
One issue will likely come from making those tables read only once done. I think just deep copy them
The other possible performance issue is atomic operations while writing to the table. I think just put a mutex on the whole table for now


Need to sort out what Im doing with the databases. The database being written to needs a mutex. Ideally the other databases wouldn't.
I think this can be achieved by using an arc for the list of databases. Each database may also need its own arc though
What about having a mutex which is either held by the add-to-list function or by the analyse function (which would take a read only ref)


Sorted threading issues, now need to show the existing database being used in the next iterations. Make the first database all zeros,
then make entries in subsequent databases the sum of previous ones. Need to create a struct to handle looking up a database entry and ensuring data is contiguous

Create database in function (Arc)
for num_stones in 2..(max_stones + 1)
    Arc clones
    for each thread
        analyse_batch
