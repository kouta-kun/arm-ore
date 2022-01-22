#ifndef __MEMALLOC_HPP
#define __MEMALLOC_HPP
#include <stdint.h>
#include <stddef.h>

enum Status : unsigned char {
  FREE = 1,
  USED = 2,
  BOUNDARY = 3
};

class BitmapAllocator {
  size_t blocks;
  size_t factor;
  void *arena;
  Status *bitmap;

public:

  BitmapAllocator(size_t factor, size_t arenasize);
  void *allocate(size_t size);
  void free(void *ptr);
};
#endif
