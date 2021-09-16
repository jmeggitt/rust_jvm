#include <stdarg.h>
#include <stdio.h>

#ifdef linux
#define JNIEXPORT
#else
#define JNIEXPORT __declspec(dllexport)
#endif

// TODO: Implement these in Rust
extern int jio_vsnprintf(char *str, size_t count, const char *fmt, va_list args);
extern int jio_vfprintf(FILE* f, const char *fmt, va_list args);

JNIEXPORT int jio_printf(const char *fmt, ...) {
    int len;
    va_list args;
    va_start(args, fmt);
    len = jio_vfprintf(stdout, fmt, args);
    va_end(args);
    return len;
}

JNIEXPORT int jio_snprintf(char *str, size_t count, const char *fmt, ...) {
    va_list args;
    int len;
    va_start(args, fmt);
    len = jio_vsnprintf(str, count, fmt, args);
    va_end(args);
    return len;
}

JNIEXPORT int jio_fprintf(FILE* f, const char *fmt, ...) {
    int len;
    va_list args;
    va_start(args, fmt);
    len = jio_vfprintf(f, fmt, args);
    va_end(args);
    return len;
}
