def split_turns:
  . as $ids |
  ($ids|indices([201,35,560,652,714,28])|first) as $assistant |
  ($ids|indices([201,55,2728,28])|last) as $user |
  {fact:$ids[:($assistant+6)], question:$ids[$user:]};
$live[0].turns[1].prompt_ids as $live_ids |
$live[0].turns[0].reply_ids as $generated |
[[$live_ids,"client"],[$dev[0].prompt_ids,"fixture"]][] as $source |
($source[0]|split_turns) as $frame |
[[$generated,"generated"],[[1],"empty"]][] as $history |
{name:($source[1]+"-"+$history[1]),prompt:($frame.fact+$history[0]+$frame.question)}
