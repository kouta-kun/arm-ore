#include "memalloc.hpp"
#include "syscall.hpp"

BitmapAllocator::BitmapAllocator(size_t factor, size_t arenasize) {
  this->factor = factor;
  this->blocks = arenasize / factor;
    
  this->arena = reinterpret_cast<void*>(SYSCALL(0x60, arenasize));
  this->bitmap = reinterpret_cast<Status*>(SYSCALL(0x60, blocks));

  for(size_t i = 0; i < blocks; i++)
    bitmap[i] = FREE;
}

void * BitmapAllocator::allocate(size_t size) {
  if(size == 0) return nullptr;

  size_t reqBlocks = size / factor + ((size % factor) > 0 ? 1 : 0);
  size_t location = 0;
  while(location <= blocks - reqBlocks) {
    size_t available = 0;
    for(size_t i = 0; i < reqBlocks; i++) {
      if(bitmap[i + location] != FREE)
	break;
      available++;
    }

    if(available == reqBlocks) {
      void *ptr = (void*)((uint8_t*)arena + (factor * location));
      bitmap[location] = BOUNDARY;
      for(size_t i = 1; i < reqBlocks; i++) {
	bitmap[location+i] = USED;
      }
      return ptr;
    } else location = location + available + 1;
  }
  return nullptr;
}

void BitmapAllocator::free(void *ptr) {
  if(ptr == nullptr) {
    return;
  }

  size_t arenaptr = (size_t)((uint8_t*)ptr - (uint8_t*)arena);
  size_t block = arenaptr / factor;

  bitmap[block] = FREE;
  while(block < blocks && bitmap[block] == USED) {
    bitmap[block] = FREE;
    block++;
  }
}
