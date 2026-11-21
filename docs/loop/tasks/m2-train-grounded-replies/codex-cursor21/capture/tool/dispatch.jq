[$manifest[0][] as $m | ($spans[0][] | select(.id==$m.id)) as $s | ($ts[0][] | select(.id==$m.id) | .record) as $t |
 if $t.turns[0].prompt_ids != $s.first_prompt_ids then error("first prompt mismatch: "+$m.id) else . end |
 if ($t.turns|length) == 2 then
  if $t.turns[0].reply_ids[-1] != 1 then error("no EOS: "+$m.id) else . end |
  ($t.turns[0].prompt_ids + $t.turns[0].reply_ids) as $history |
  if $t.turns[1].prompt_ids[0:($history|length)] != $history then error("history mismatch: "+$m.id) else . end |
  {id:$m.id,role:$m.role,prompt:$t.turns[1].prompt_ids,ordinary_reply:$t.turns[1].reply_ids,job_value_start:$s.job_value_start}
 else empty end] | {job_form:1223,home_form:1553,rows:.}
