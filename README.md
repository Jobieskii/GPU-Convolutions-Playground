This tool uses one parameter, path to a `.yaml` file containing a program.

## example program – Game Of Life
```
screen: [3840, 2160]

type: val
matrix:
 - [1.,  1., 1,]
 - [1., -9., 1.]
 - [1.,  1., 1.]
fun: >
  if (x == -7. || x == -6. || x == 3.)
    return 1.;
  return 0.;
```