#include <stddef.h>
#include <stdint.h>
#include "stdlib/syscall.hpp"
#include "stdlib/memalloc.hpp"
#include "stdlib/console.hpp"
#include "stdlib/fileio.hpp"

char * file_path = "./README.TXT";

size_t strlen(char* str) {
  size_t sz = 0;
  while(*(str++))sz++;
  return sz;
}

extern "C"
void main() {
  BitmapAllocator ba(64, 1024*1024*64);

  size_t fs = file_size(file_path);

  char *file_content = reinterpret_cast<char*>(ba.allocate(fs+1));

  read_file(file_path, 0, fs, reinterpret_cast<uint8_t*>(file_content));

  file_content[fs] = 0;
  println(file_content);
}
