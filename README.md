# Prompt Builder (pb)

The Prompt Builder CLI (`pb`) allows you to manage a collection of file paths. This collection is persistently stored and can be used to easily print the content of all managed files.

### Commands

*   **`pb add <file_pattern_1> [file_pattern_2 ...]`**: Adds files to the collection.
    *   Accepts one or more glob patterns to match files.
    *   It respects `.gitignore` rules by default.
    *   It explicitly ignores `*.lock` files.
    *   Duplicate files (based on their absolute path) are not added.
*   **`pb list`**: Lists all files currently in the collection, showing both their relative and absolute paths.
*   **`pb clear`**: Removes all files from the collection.
*   **`pb print`**: Prints the content of all files in the collection.
    *   The output is formatted with XML-like tags:
        *   A root `<files>` tag.
        *   Each file's content is wrapped in a `<file path="relative/path/to/file">...</file>` tag.
*   **`pb info`**: Displays the path to the `state.json` file where the collection of files is stored.

### State Management

The CLI maintains its state (the list of file paths) in a `state.json` file.
*   On macOS, this file is typically located at: `~/Library/Application Support/org.sweb.PromptBuilder/state.json`
*   On Linux, this file is typically located at: `~/.config/PromptBuilder/state.json`