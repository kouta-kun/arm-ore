#ifndef __FILEIO_HPP
#define __FILEIO_HPP
#include "syscall.hpp"
#include <stdint.h>

size_t file_count();

size_t filename_length(size_t file_index);

size_t filename_char(size_t file_index, size_t char_index);

size_t file_size(char* filename);

size_t read_file(char* filename, size_t offset, size_t size, uint8_t* output);

#endif
