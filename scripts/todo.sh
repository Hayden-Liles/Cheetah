#!/bin/bash

# Simple script to manage the TODO.md file

TODO_FILE="TODO.md"

# Check if the TODO file exists
if [ ! -f "$TODO_FILE" ]; then
    echo "Error: $TODO_FILE not found!"
    exit 1
fi

# Function to list all tasks
list_tasks() {
    echo "Current tasks:"
    echo "=============="
    grep -n "- \[ \]" "$TODO_FILE" | sed 's/- \[ \]/☐/g'
    echo ""
    echo "Completed tasks:"
    echo "================"
    grep -n "- \[x\]" "$TODO_FILE" | sed 's/- \[x\]/✓/g'
}

# Function to mark a task as completed
complete_task() {
    LINE_NUM=$1
    if [ -z "$LINE_NUM" ]; then
        echo "Error: Please provide a line number"
        exit 1
    fi
    
    # Check if the line exists and contains an incomplete task
    if grep -q "^$LINE_NUM:- \[ \]" <(grep -n "- \[ \]" "$TODO_FILE"); then
        # Extract the line number in the actual file
        ACTUAL_LINE=$(grep -n "- \[ \]" "$TODO_FILE" | grep "^$LINE_NUM:" | cut -d ":" -f 2)
        # Replace the [ ] with [x]
        sed -i "${ACTUAL_LINE}s/- \[ \]/- \[x\]/" "$TODO_FILE"
        echo "Task marked as completed!"
    else
        echo "Error: Invalid task number or task already completed"
    fi
}

# Function to add a new task
add_task() {
    TASK=$1
    SECTION=$2
    
    if [ -z "$TASK" ]; then
        echo "Error: Please provide a task description"
        exit 1
    fi
    
    if [ -z "$SECTION" ]; then
        # Default to adding to High Priority section
        SECTION="High Priority"
    fi
    
    # Find the section
    SECTION_LINE=$(grep -n "^### $SECTION" "$TODO_FILE" | cut -d ":" -f 1)
    
    if [ -z "$SECTION_LINE" ]; then
        echo "Error: Section '$SECTION' not found"
        echo "Available sections:"
        grep "^### " "$TODO_FILE" | sed 's/### //g'
        exit 1
    fi
    
    # Find the next empty line after the section
    NEXT_EMPTY_LINE=$((SECTION_LINE + 1))
    while [ "$(sed -n "${NEXT_EMPTY_LINE}p" "$TODO_FILE" | grep -c "^$")" -eq 0 ]; do
        NEXT_EMPTY_LINE=$((NEXT_EMPTY_LINE + 1))
    done
    
    # Insert the new task before the empty line
    sed -i "${NEXT_EMPTY_LINE}i- [ ] $TASK" "$TODO_FILE"
    echo "Task added to $SECTION!"
}

# Main command processing
case "$1" in
    list|ls)
        list_tasks
        ;;
    complete|done)
        complete_task "$2"
        ;;
    add|new)
        add_task "$2" "$3"
        ;;
    *)
        echo "Usage: $0 {list|complete|add} [args]"
        echo ""
        echo "Commands:"
        echo "  list                List all tasks"
        echo "  complete <number>   Mark task as completed"
        echo "  add \"task\" [section] Add a new task to the specified section"
        exit 1
        ;;
esac

exit 0
