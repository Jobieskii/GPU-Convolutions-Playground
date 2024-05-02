# Convolutions Playground
A tool for easy experimentation with visuals generetad by convolution with a 2d matrix. It uses a GPU for much faster simulation.

This tool uses one parameter, path to a `.yaml` file containing a program.

## Controls
 - Space - fill randomly with red at 0 or 1
 - `x` - fill randomly with red with value between 0 and 1
 - 'c' - fill with black
 - `1`..`-` - number keys set speed of simulation ('1' is every 32 frames, '6' every frame, '-' 32 steps every frame)
 - '=' - pause
 - 'r', 'g', 'b', 'w' - set paint color ('w', white)
 - LeftMouse - paint with selected color
 - 'q'/ESC - quit


## Types of programs
All programs must have: `type`, `screen` (width, height).
### Val
Each cell contains one float value. 

Arguments
 - `kernel` - an NxN matrix
 - `fun` - glsl function. 
  Has two arguments: `x` - value of convolution with kernel, `prev` - previous value. 
 - `edge` can be:
   - `wrap` - wrap around edges
   - `clamp` - clamp (x,y) when counting neighbors
   - _float value_ - some value

### Rgb
Each cell contains three float values.

Arguments
 - `kernel` an NxN matrix
 - `fun` - glsl function
 Has two arguments `v` and `prev`, both a vec3. Must return a vec3.
 - `edge` same as 'Val' except value must be a tuple

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

Other examples can be found in 'programs/'.
