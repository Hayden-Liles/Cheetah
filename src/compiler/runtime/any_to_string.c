// any_to_string.c - Runtime helper for converting Any type to string

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * Attempts to convert a value of unknown type to a string
 * This is a heuristic function that tries to determine the type at runtime
 * 
 * @param p Pointer to the value
 * @return A newly allocated string representation
 */
char *any_to_string(void *p) {
    char buf[64];
    
    // Handle null pointer
    if (!p) {
        return strdup("None");
    }
    
    // Try to interpret as an integer
    long v = *(long*)p;
    sprintf(buf, "%ld", v);
    if (v != 0) {
        return strdup(buf);
    }
    
    // Try to interpret as a string
    const char *str = (const char*)p;
    if (str && str[0] != '\0') {
        return strdup(str);
    }
    
    // If all else fails, return a generic string
    return strdup("?");
}
