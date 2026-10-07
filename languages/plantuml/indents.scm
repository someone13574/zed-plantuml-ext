(_ "{" "}" @end) @indent
(_ "(" ")" @end) @indent

(frame_block "end" @end) @indent
(fork_block "end" @end) @indent
(split_block "end" @end) @indent
(if_block "endif" @end) @indent
(while_block "endwhile" @end) @indent
(switch_block "endswitch" @end) @indent
(repeat_block "repeat" "repeat" @end) @indent
(box_block "end box" @end) @indent
(note_statement "end note" @end) @indent
(reference "end ref" @end) @indent

[
  (else_clause)
  (elseif_clause)
  (case_clause)
  (fork_again)
  (split_again)
] @outdent
