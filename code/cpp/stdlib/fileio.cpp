#include "fileio.hpp"

size_t file_count() {
  return SYSCALL(0);
}

size_t filename_length(size_t file_index) {
  return SYSCALL(1, file_index);
}

size_t filename_char(size_t file_index, size_t char_index) {
  return SYSCALL(2, file_index, char_index);
}

size_t file_size(char *filename) {
  return SYSCALL(3, reinterpret_cast<size_t>(filename));
}

size_t read_file(char* filename, size_t offset, size_t size, uint8_t *output) {
  return SYSCALL(4, reinterpret_cast<size_t>(filename), offset, size, reinterpret_cast<size_t>(output));
}
