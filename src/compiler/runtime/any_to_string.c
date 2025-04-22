// any_to_string.c - Runtime helper for converting Any type to string

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <math.h>

// Structure for a raw list (must match the Rust definition)
typedef struct {
    void **data;
    long length;
    long capacity;
} RawList;

/**
 * Attempts to convert a value of unknown type to a string
 * This is a heuristic function that tries to determine the type at runtime
 *
 * @param p Pointer to the value
 * @return A newly allocated string representation
 */
char *any_to_string(void *p) {
    char buf[256];

    // Handle null pointer
    if (!p) {
        return strdup("None");
    }

    // Try to interpret as a string first
    const char *str = (const char*)p;
    if (str && str[0] != '\0') {
        // Check if it's a printable string
        int is_printable = 1;
        for (int i = 0; str[i] != '\0' && i < 20; i++) {
            if (str[i] < 32 || str[i] > 126) {
                is_printable = 0;
                break;
            }
        }

        if (is_printable) {
            sprintf(buf, "\"%s\"", str);
            return strdup(buf);
        }
    }

    // Try to interpret as an integer
    int64_t i64 = *(int64_t*)p;
    sprintf(buf, "%ld", i64);
    return strdup(buf);
}
