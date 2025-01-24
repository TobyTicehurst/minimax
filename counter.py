from anytree import Node, RenderTree
import math

counter = 0

def add_nodes(parent, depth, remaining):
    global counter
    for i in reversed(range(0, remaining + 1)):
        child = Node(str(i), parent=parent)
        if depth > 1:
            add_nodes(child, depth - 1, remaining - i)
        else:
            Node(str(counter), parent=child)
            counter += 1

def total_leaves(depth, stones):
    return math.factorial(depth + stones) // (math.factorial(depth) * math.factorial(stones))

def leaves_to_skip(depth, stones):
    return (math.factorial(depth + stones + 1) // (math.factorial(depth + 1) * math.factorial(stones)))

def all_leaves_to_skip(total_depth, total_stones, stones_list, cache):
    count = 0
    depth = total_depth
    remaining = total_stones - 1
    for depth in range(0, total_depth):
        stones = stones_list[depth]
        if stones == remaining:
            return count
        remaining -= stones
        # print(f"{count}, {depth}, {remaining - 1}")
        count += cache[total_depth - depth - 1][remaining]
    
    return count

def get_element(root, stones_list):
    node = root
    for stones in stones_list:
        node = list(reversed(node.children))[stones]
    
    return int(node.children[0].name)

max_stones = 2
cache = [[0 for i in range(max_stones)] for j in range(12)]

for i in range(0, 12):
    for j in range(0, max_stones):
        cache[i][j] = leaves_to_skip(i, j)

depth = 12
stones = 2
stones_list = [0] * depth
root = Node("Tree")
add_nodes(root, depth, stones)

for pre, _, node in RenderTree(root):
    print(f"{pre}{node.name}")

print(f"Counting leaves: {len(root.leaves)}")
print(f"Mathsing leaves: {total_leaves(depth, stones)}")
print(f"Test2: {all_leaves_to_skip(depth, stones, stones_list, cache)}")
print(f"{get_element(root, stones_list)}")

for line in cache:
    print_string = ""
    for element in line:
        print_string += f"{' ' * (9-len(str(element)))}{element} "
    print(print_string)