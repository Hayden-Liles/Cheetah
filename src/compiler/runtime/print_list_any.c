// print_list_any.c - Runtime helper for printing lists with Any type elements

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Forward declaration for any_to_string function
extern char *any_to_string(void *p);

// Structure for a raw list (must match the Rust definition)
typedef struct {
    void **data;
    long length;
    long capacity;
} RawList;

/**
 * Prints a list with Any type elements
 * This function handles heterogeneous lists by using any_to_string for each element
 * 
 * @param list Pointer to the RawList
 */
void print_list_any(RawList *list) {
    if (!list) {
        printf("None");
        return;
    }
    
    printf("[");
    
    for (long i = 0; i < list->length; i++) {
        void *elem = list->data[i];
        
        // Convert the element to a string using any_to_string
        char *str = any_to_string(elem);
        
        // Print the string
        printf("%s", str);
        
        // Free the string
        free(str);
        
        // Print comma if not the last element
        if (i < list->length - 1) {
            printf(", ");
        }
    }
    
    printf("]");
}
