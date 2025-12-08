/**
 * SimplicityHL (Simfony) Syntax Highlighting Mode for CodeMirror
 * Based on the official VSCode extension: https://marketplace.visualstudio.com/items?itemName=Blockstream.simplicityhl
 * 
 * In VSCode:
 * - jet/witness/param = entity.name.namespace (namespace color)
 * - function_name after :: = entity.name.function (function color)
 * 
 * Token types used:
 * - comment: gray
 * - keyword: pink (fn, let, match, if, else, etc.)
 * - def: green (function names in definitions)
 * - variable: white (regular identifiers)
 * - variable-2: orange (function calls, parameters)
 * - variable-3: purple (constants, UPPERCASE)
 * - builtin: cyan (types, built-in functions)
 * - atom: purple (true, false, None, Left, Right, Some)
 * - string: yellow
 * - number: purple
 * - operator: pink
 * - meta: pink (macros like assert!)
 * - namespace: special handling for jet::, witness::, param::
 */

(function(mod) {
    if (typeof exports == "object" && typeof module == "object") // CommonJS
        mod(require("../../lib/codemirror"), require("../../addon/mode/simple"));
    else if (typeof define == "function" && define.amd) // AMD
        define(["../../lib/codemirror", "../../addon/mode/simple"], mod);
    else // Plain browser env
        mod(CodeMirror);
})(function(CodeMirror) {
    "use strict";

    // Custom token function for namespace::identifier patterns
    function tokenNamespace(stream, state) {
        // Check if we're at jet::, witness::, or param::
        if (stream.match(/\b(jet|witness|param)::/)) {
            state.nextToken = 'namespacedId';
            return 'builtin'; // namespace part gets cyan
        }
        return null;
    }

    CodeMirror.defineSimpleMode("simplicityhl", {
        start: [
            // === COMMENTS (first priority) ===
            {regex: /\/\/.*/, token: "comment"},
            {regex: /\/\*/, token: "comment", next: "comment"},
            
            // === MACROS (assert!, panic!, etc.) ===
            {regex: /\b[a-z_][a-zA-Z0-9_]*!/, token: "meta"},
            
            // === KEYWORDS ===
            {regex: /\b(fn|let|match|if|else|while|for|return|type|mod|const)\b/, token: "keyword"},
            
            // === BOOLEAN & SPECIAL CONSTANTS ===
            {regex: /\b(true|false|None)\b/, token: "atom"},
            
            // === EITHER/OPTION VARIANTS ===
            {regex: /\b(Left|Right|Some)\b/, token: "atom"},
            
            // === TYPES - Cyan ===
            {regex: /\b(u1|u2|u4|u8|u16|u32|u64|u128|u256|i8|i16|i32|i64|bool)\b/, token: "builtin"},
            {regex: /\b(Either|Option|List)\b/, token: "builtin"},
            {regex: /\b(Ctx8|Pubkey|Message64|Message|Signature|Scalar|Fe|Gej|Ge|Point)\b/, token: "builtin"},
            {regex: /\b(Height|Time|Distance|Duration|Lock|Outpoint)\b/, token: "builtin"},
            {regex: /\b(Confidential1|ExplicitAsset|Asset1|ExplicitAmount|Amount1|ExplicitNonce|Nonce|TokenAmount1)\b/, token: "builtin"},
            {regex: /\b[A-Z][a-zA-Z0-9_]*\b/, token: "builtin"},
            
            // === BUILT-IN FUNCTIONS ===
            {regex: /\b(unwrap|unwrap_left|unwrap_right|for_while|is_none|array_fold|into|fold|dbg)\b/, token: "builtin"},
            
            // === NAMESPACE::IDENTIFIER PATTERNS ===
            // jet:: (cyan) + function_name (green)
            {regex: /\b(jet)(::)([a-z_][a-zA-Z0-9_]*)/, token: ["builtin", "operator", "def"]},
            
            // witness:: (cyan) + CONSTANT (purple)
            {regex: /\b(witness)(::)([A-Z_][A-Z0-9_]*)/, token: ["builtin", "operator", "variable-3"]},
            
            // param:: (cyan) + name (green)
            {regex: /\b(param)(::)([a-z_][a-zA-Z0-9_]*)/, token: ["builtin", "operator", "def"]},
            
            // === NUMBERS ===
            {regex: /\b0x[0-9a-fA-F_]+\b/, token: "number"},
            {regex: /\b0b[01_]+\b/, token: "number"},
            {regex: /\b[0-9][0-9_]*\b/, token: "number"},
            
            // === STRINGS ===
            {regex: /"(?:[^\\"]|\\.)*?"/, token: "string"},
            
            // === FUNCTION DEFINITIONS ===
            {regex: /\b(fn)\s+/, token: "keyword", next: "functionName"},
            
            // === FUNCTION CALLS (before general variables) ===
            {regex: /\b[a-z_][a-zA-Z0-9_]*(?=\s*\()/, token: "variable-2"},
            
            // === OPERATORS ===
            {regex: /->|=>|::|==|!=|<=|>=|&&|\|\|/, token: "operator"},
            {regex: /[+\-*\/%&|^!~<>=:]/, token: "operator"},
            
            // === BRACKETS & PUNCTUATION ===
            {regex: /[\{\[\(]/, indent: true},
            {regex: /[\}\]\)]/, dedent: true},
            {regex: /[;,.]/, token: null},
            
            // === VARIABLES ===
            {regex: /\b[a-z_][a-zA-Z0-9_]*\b/, token: "variable"}
        ],
        
        // State for capturing function names after "fn "
        functionName: [
            {regex: /[a-zA-Z_][a-zA-Z0-9_]*/, token: "def", next: "start"},
            {regex: /\s+/, token: null}
        ],
        
        // Multi-line comment state
        comment: [
            {regex: /.*?\*\//, token: "comment", next: "start"},
            {regex: /.*/, token: "comment"}
        ],
        
        // Language metadata
        meta: {
            lineComment: "//",
            blockCommentStart: "/*",
            blockCommentEnd: "*/",
            fold: "brace",
            electricChars: "{}[]",
            closeBrackets: "()[]{}''\"\"``"
        }
    });

    // Register MIME types
    CodeMirror.defineMIME("text/x-simplicityhl", "simplicityhl");
    CodeMirror.defineMIME("text/x-simfony", "simplicityhl");
    CodeMirror.defineMIME("text/x-simf", "simplicityhl");
});

