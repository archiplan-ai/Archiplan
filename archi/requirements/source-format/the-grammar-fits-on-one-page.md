---
kind: functional
origin: intent
satisfied-by: [Compiler.Parser]
deferred:
---

# The grammar fits on one page

The whole surface language is one page of EBNF over the lexer's token stream, and
the parser accepts exactly it: imports first, then top-level items; type
definitions only at top level; ports only inside `def node` blocks. Lexical and
grammatical failures are `E_PARSE`, located at the offending token, and parse
errors from several files are collected rather than first-error-only
(`located-diagnostics`).

```ebnf
file         = { import_decl } { top_item }

import_decl  = "import" module_path [ "(" ident { "," ident } ")" ] NL
module_path  = ident { "." ident }

top_item     = def_view | def_rel | def_conn | def_node | open_block | edge_stmt | app_stmt

def_view     = "def" "view" ident NL
def_rel      = "def" "rel" [ "trans" ] ident ":=" slot_pat rel_arrow slot_pat NL
rel_arrow    = "->" | "<->"

def_conn     = "def" "conn" ident ":=" slot_pat lanes slot_pat NL
lanes        = "<->" [ slot_pat ]                        (* undirected; optional single carried slot *)
             | "->" [ slot_pat ] [ "," "<-" slot_pat ]   (* directed; forward and/or reverse carried slots *)

def_node     = "def" "node" path ( NL | ":" NL INDENT node_body DEDENT )
node_body    = { port_decl | def_node | open_block | edge_stmt | app_stmt }
port_decl    = "port" ident NL

open_block   = "open" path ":" NL INDENT open_body DEDENT
open_body    = { def_node | open_block | edge_stmt | app_stmt }   (* no port_decl *)

edge_stmt    = path type_ref path [ views ] NL           (* rel or conn edge, decided by the type's kind *)
type_ref     = ident [ "(" carrier_arg { "," carrier_arg } ")" ]
carrier_arg  = [ "->" | "<-" ] path                      (* a concrete carried node *)
views        = "in" ident { "," ident }

app_stmt     = path [ "(" bare_pat ")" ] "=" ident "." ident NL   (* outer[(route)] = Child.port *)

slot_pat     = "*" | path | "(" bare_pat ")"
bare_pat     = "*" | path | path ident "*"               (* any | exact | classified *)
path         = ident { "." ident }
```

The one ambiguity is carrier-vs-target after a lane arrow, and it resolves by
lookahead: a pattern followed by another pattern (or by the `,` lane separator) was
the lane's carried slot; a pattern followed by end of line was the target. Blocks
opened by `:` must be non-empty; an `in` views clause is legal only after an edge.

## System Context

`NL`, `INDENT` and `DEDENT` arrive from the offside-rule lexer
(`the-offside-rule-shapes-the-source`), which is what keeps the grammar itself
context-free. The grammar is the agents' contract for authoring: everything an
agent can write is derivable from this page, and what parses is still only
candidate input for name resolution and the engine's semantics
(`one-semantic-authority`).

## Satisfy

`Compiler.Parser` (recursive descent over the token stream; the context-free
constraints — top-level-only type defs, no ports in `open`, non-empty blocks,
views-after-edges — live here, with spans pointing at the offending token).

- test — parser::all_conn_lane_forms_parse, parser::rel_defs_parse, parser::edges_parse_with_carriers_and_views
- test — parser::conn_lane_lookahead_separates_carrier_from_target
- test — parser::type_defs_are_top_level_only, parser::empty_blocks_are_rejected, parser::views_clause_only_after_edges
- test — parser::imports_parse_and_precede_items
- test — parser::error_spans_locate_the_offender
- test — source_e2e::parse_errors_from_several_files_are_collected
