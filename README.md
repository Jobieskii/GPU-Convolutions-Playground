# Convolutions Playground
This tool uses one parameter, path to a `.yaml` file containing a program.

## Types of programs
All programs must have: `type`, `screen` (width, height).
### Val
Each cell contains one float value. 

Arguments
 - `kernel` - a 3x3 matrix
 - `fun` - glsl function. 
  Has two arguments: `x` - value of convolution with kernel, `prev` - previous value. 
 - `edge` can be:
   - `wrap` - wrap around edges
   - `clamp` - clamp (x,y) when counting neighbors
   - _float value_ - some value

## example program – Game Of Life
```
screen: [3840, 2160]

type: val
edge: wrap
kernel:
 - [1.,  1., 1,]
 - [1., -9., 1.]
 - [1.,  1., 1.]
fun: >
  if (x == -7. || x == -6. || x == 3.)
    return 1.;
  return 0.;
```