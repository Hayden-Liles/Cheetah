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
    TASKS=$(grep -n -- "- \[ \]" "$TODO_FILE")
    if [ -z "$TASKS" ]; then
        echo "No pending tasks!"
    else
        # Number the tasks for easier reference
        echo "$TASKS" | nl -w2 -s". " | sed 's/:[0-9]*:- \[ \]/. ☐ /g'
    fi

    echo ""
    echo "Completed tasks:"
    echo "================"
    COMPLETED=$(grep -n -- "- \[x\]" "$TODO_FILE")
    if [ -z "$COMPLETED" ]; then
        echo "No completed tasks yet!"
    else
        echo "$COMPLETED" | sed 's/[0-9]*:- \[x\]/✓/g'
    fi
}

# Function to mark a task as completed
complete_task() {
    TASK_NUM=$1
    if [ -z "$TASK_NUM" ]; then
        echo "Error: Please provide a task number"
        exit 1
    fi

    # Get all incomplete tasks
    TASKS=$(grep -n -- "- \[ \]" "$TODO_FILE")

    # Count the number of tasks
    TASK_COUNT=$(echo "$TASKS" | wc -l)

    # Check if the task number is valid
    if [ "$TASK_NUM" -gt 0 ] && [ "$TASK_NUM" -le "$TASK_COUNT" ]; then
        # Get the line number of the task in the file
        LINE_NUM=$(echo "$TASKS" | sed -n "${TASK_NUM}p" | cut -d ":" -f 1)

        # Replace the [ ] with [x]
        sed -i "${LINE_NUM}s/- \[ \]/- \[x\]/" "$TODO_FILE"
        echo "Task marked as completed!"
    else
        echo "Error: Invalid task number. Please choose a number between 1 and $TASK_COUNT"
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
