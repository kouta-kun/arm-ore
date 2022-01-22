#include "graphics.hpp"
#include "syscall.hpp"
#define STORE_ATTR(attribute) this->attribute = attribute

Vertex::Vertex(float x, float y, float z, float w, float r, float g, float b, float a)
{
        STORE_ATTR(x);
        STORE_ATTR(y);
        STORE_ATTR(z);
        STORE_ATTR(w);

        STORE_ATTR(r);
        STORE_ATTR(g);
        STORE_ATTR(b);
        STORE_ATTR(a);
}

void submit_drawlist(Vertex *vertexList, size_t vertexCount) {
    SYSCALL(0x160, reinterpret_cast<size_t>(vertexList), vertexCount);
}