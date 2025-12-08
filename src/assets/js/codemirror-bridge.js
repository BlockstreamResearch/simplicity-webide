/**
 * CodeMirror Bridge for Simplicity Web IDE
 * This file bridges CodeMirror with the Leptos/WASM application
 */

window.SimplicityEditor = (function() {
    let editorInstance = null;

    return {
        /**
         * Initialize CodeMirror editor on a textarea element
         * @param {string} textareaId - The ID of the textarea element
         * @param {string} initialValue - Initial code content
         * @returns {boolean} Success status
         */
        init: function(textareaId, initialValue) {
            try {
                const textarea = document.getElementById(textareaId);
                if (!textarea) {
                    console.error('Textarea not found:', textareaId);
                    return false;
                }

                // Create CodeMirror instance
                editorInstance = CodeMirror.fromTextArea(textarea, {
                    mode: 'simplicityhl',
                    theme: 'simplicity',
                    lineNumbers: true,
                    matchBrackets: true,
                    autoCloseBrackets: true,
                    indentUnit: 4,
                    tabSize: 4,
                    indentWithTabs: false,
                    lineWrapping: false,
                    extraKeys: {
                        "Tab": function(cm) {
                            cm.replaceSelection("    ", "end");
                        },
                        "Shift-Tab": function(cm) {
                            // Unindent
                            const cursor = cm.getCursor();
                            const line = cm.getLine(cursor.line);
                            if (line.startsWith("    ")) {
                                cm.replaceRange("", 
                                    { line: cursor.line, ch: 0 },
                                    { line: cursor.line, ch: 4 }
                                );
                            }
                        },
                        "Ctrl-Enter": function(cm) {
                            // Trigger run - dispatch custom event
                            window.dispatchEvent(new CustomEvent('codemirror-ctrl-enter'));
                        }
                    }
                });

                // Set initial value
                if (initialValue) {
                    editorInstance.setValue(initialValue);
                }

                // Listen for changes and update the hidden textarea
                editorInstance.on('change', function(cm) {
                    textarea.value = cm.getValue();
                    // Dispatch input event so Leptos can detect the change
                    textarea.dispatchEvent(new Event('input', { bubbles: true }));
                });

                console.log('CodeMirror initialized successfully');
                return true;
            } catch (error) {
                console.error('Failed to initialize CodeMirror:', error);
                return false;
            }
        },

        /**
         * Get the current editor content
         * @returns {string|null} Current content or null if editor not initialized
         */
        getValue: function() {
            return editorInstance ? editorInstance.getValue() : null;
        },

        /**
         * Set the editor content
         * @param {string} value - New content
         */
        setValue: function(value) {
            if (editorInstance) {
                editorInstance.setValue(value || '');
            }
        },

        /**
         * Refresh the editor (useful after visibility changes)
         */
        refresh: function() {
            if (editorInstance) {
                setTimeout(function() {
                    editorInstance.refresh();
                }, 1);
            }
        },

        /**
         * Focus the editor
         */
        focus: function() {
            if (editorInstance) {
                editorInstance.focus();
            }
        },

        /**
         * Get the editor instance
         * @returns {object|null} CodeMirror instance or null
         */
        getInstance: function() {
            return editorInstance;
        }
    };
})();

