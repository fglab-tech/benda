pred = (λx1 λx2
        let nx1=(== x1 0)
        let nx2=(== x2 0)
        let res=(&(&(| x1 x2) (| x1 nx2)) (| nx1 x2))
        (res))

main = (pred(0, 1)(0, 1))
