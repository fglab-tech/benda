
# List clear:
# List l -> list l
# clears all elements from list l. This is equivalent to initializing an empty list.
clear = @l[]

# List concat:
# List l -> List l
# combines two lists (l1, l2) from left to right.
concat = @l1 @ l2
match l1 {
    List/Cons: (List/Cons l1.head(concat l1.tail l2))
    List/Nil: l2
}

# List add_front:
# List l -> List l
# adds a non-List element e to the front of list l.
add_front = @l @ e
match l {
    List/Cons: (List/Cons e l)
    List/Nil: (List/Cons e List/Nil)
}

# List append (add_back):
# List l -> List l
# adds a non-list element e to the back of list l.
append = @l @ e
(concat l(List/Cons e List/Nil))

# list sum:
# List l -> uint
# returns the sum of all items in the list.
sum = @l
match l {
    List/Cons: (+ l.head(sum l.tail))
    List/Nil: 0
}

# List reverse:
# List l -> List l
# reverses the order of elements in list l.
reverse.aux = @acc @ l
match l {
    List/Nil: acc
    List/Cons: (reverse.aux(List/Cons l.head acc) l.tail)
}
reverse = @l
(reverse.aux[] l)

# List length:
# List l -> uint
# returns the number of elements in list l.
len = @l
match l {
    List/Nil: 0
    List/Cons: (+ 1 (len l.tail))
}

# List count:
# List l -> uint -> uint
# returns the number of instances of Some s in list l.
count.aux = @acc @ l @ s
match l {
    List/Nil: acc
    List/Cons: use acc = switch(== l.head s) {
        0: acc
        _: (+ acc 1)
    }
    (count.aux acc l.tail s)
}
count = @l @ s
(count.aux 0 l s)

# List index:
# List l -> Some s
# returns the value of a specific list index i, or * if the index doesn't exist.
index = @l @ i
match l {
    List/Cons:
    switch i {
        0: l.head
        _: (index l.tail(i-1))
    }
    List/Nil: *
}

# List head:
# List l -> Some s
# returns the first item in the list, or [] if the list is empty.
head = @l
match l {
    List/Cons: l.head
    List/Nil: []
}

# List pop_front:
# List l -> List l
# removes and discards the first item of list l.
# The new list is returned, or [] if the list is empty.
pop_front = @l
match l {
    List/Cons: l.tail
    List/Nil: []
}

# List pop_back:
# List l -> List l
# removes and discards the the last item of list l.
pop_back(List/Nil) = List/Nil
pop_back(List/Cons x List/Nil) = List/Nil
pop_back(List/Cons head tail) = (List/Cons head(pop_back tail))

# List remove:
# List l -> Some s -> List l
# removes the first occurrence of element e from list l.
remove = @l @ s
match l {
    List/Cons:
    switch(== l.head s) {
        0: (List/Cons l.head(remove l.tail s))
        _: l.tail
    }
    List/Nil: List/Nil
}

# List split:
# list l -> uint i -> (List l, list l)
# splits list l into two lists (l1, l2) at index i.
# the second list takes the element at index i during the split.
split = @l @ i(split.aux[] l i)

split.aux = @acc @ l @ i
match l {
    List/Cons:
    switch i {
        0: (acc, l)
        _: (split.aux(append acc l.head) l.tail i-1)
    }
    List/Nil: *
}

find_unit = @l
match l {
    List/Cons:
    switch(!= 1 (len l.head)) {
        0: (List/Cons(head l.head)(find_unit l.tail))
        _: (find_unit l.tail)
    }
    List/Nil: List/Nil
}


def check_contradiction(vars, units):
  match vars:
    case List/Cons:
      count_pos = count(units, vars.head)
      count_neg = count(units, (vars.head * -1))
      if (count_pos > 0) & (count_neg > 0):
        return -1
      else:
        return check_contradiction(vars.tail, units)
    case List/Nil:
      return 1


def simplify_clause(clause, unit):
  if count(clause, unit) > 0:
    return List/Nil
  else:
    match clause:
      case List/Cons:
        if clause.head != (unit * -1):
          return List/Cons(clause.head, simplify_clause(clause.tail, unit))
        else:
          return simplify_clause(clause.tail, unit)
      case List/Nil:
        return List/Nil


def simplify_formula(form, assi, unit, mul1, mul2):
  match form:
    case List/Cons:
      if count(form.head, (unit * mul1)) > 0:
        return List/Cons(form.head, simplify_formula(form.tail, assi, unit, mul1, mul2))
      elif count(form.head, (unit * mul2)) > 0:
        return List/Cons(simplify_clause(form.head, unit), simplify_formula(form.tail, assi, unit, mul1, mul2))
      else:
        return List/Cons(form.head, simplify_formula(form.tail, assi, unit, mul1, mul2))
    case List/Nil:
      return List/Nil


def simplify.units(form, assi, units, vars):
  match units:
    case List/Cons:
      ind = index(vars, units.head)
      vars = remove(vars, ind)

      if units.head > 0:
        assi[units.head] = 1
        return simplify_formula(form, assi, units.head, -1, 1)
      else:
        assi[(units.head * -1)] = -1
        return simplify_formula(form, assi, units.head, 1, -1)

    case List/Nil:
      return List/Nil


def unit_propagation(form, assi, vars):
  units = find_unit(form)
  cont = check_contradiction(vars, units)

  if cont == 1:
    new_form = simplify.units(form, assi, units, vars)
    # return (1, new_form, assi)
    return new_form
  elif len(units) == 0:
    return (1, form, assi)
  else:
    return -1
