#ifndef __GRAPHICS_HPP
#define __GRAPHICS_HPP
#include <stddef.h>
#include <stdint.h>

struct Vertex {
    float x;
    float y;
    float z;
    float w;

    float r;
    float g;
    float b;
    float a;

    Vertex(float x, float y, float z, float w, float r, float g, float b, float a);
};

void submit_drawlist(Vertex *vertexList, size_t vertexCount, uint16_t *indexList, size_t indexCount);
#endif