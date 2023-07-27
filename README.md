# Convolutions Playground
This tool uses one parameter, path to a `.yaml` file containing a program.

## Types of programs
### Val
Each cell contains one float value. The kernel is a 3x3 matrix. Function has two arguments, x - value of convolution with kernel, prev - previous value.

## example program – Game Of Life
```
screen: [3840, 2160]

type: val
kernel:
 - [1.,  1., 1,]
 - [1., -9., 1.]
 - [1.,  1., 1.]
fun: >
  if (x == -7. || x == -6. || x == 3.)
    return 1.;
  return 0.;
```