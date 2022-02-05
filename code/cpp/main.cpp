#include <stddef.h>
#include <stdint.h>
#include "stdlib/syscall.hpp"
#include "stdlib/memalloc.hpp"
#include "stdlib/console.hpp"
#include "stdlib/fileio.hpp"
#include "stdlib/graphics.hpp"

extern "C"
int main() {
  Vertex triangle[] = {
      {-1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0},
      {1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0},
      {0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0},
  };


  size_t frame = 0;

  uint16_t indexes[] = {0,1,2};

  while(true) {
      frame++;

      triangle[0].r = float(frame%120)/120.0f;
      triangle[1].g = float(frame%120)/120.0f;
      triangle[2].b = float(frame%120)/120.0f;

      submit_drawlist(triangle, 3, indexes, 3);
  }
}
