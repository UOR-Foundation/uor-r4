# jq -n --slurpfile grades GRADES --slurpfile predictions PREDICTIONS -f audit.jq
# Fails closed on mismatched row sets, offsets, source tokens or retained transitions.
def require($ok; $msg): if $ok then . else error($msg) end;
def firstdiff($a;$b): ([range(0; ([$a|length,$b|length]|max)) | select($a[.] != $b[.])][0] // null);
def move($a;$b): if $a==null and $b==null then "both_exact" elif $a==null then "lost" elif $b==null then "recovered" elif $b>$a then "later" elif $b<$a then "earlier" else "same" end;
def checkarm($p;$g;$name):
  $p[$name] as $a | $g[$name] as $b |
  firstdiff($a.reply;$g.gold) as $first |
  require($a.reply == $b.reply and $first == $b.first; "reply or mismatch differs: \($g.id)/\($name)") |
  require(($a.steps|length)==($a.reply|length); "step count: \($g.id)/\($name)") |
  reduce range(0;($a.steps|length)) as $i (. ;
    $a.steps[$i] as $s | ($p.prompt + $a.reply[:$i]) as $context |
    require($s.emitted==$a.reply[$i] and ($s.source==null or $context[$s.source]==$s.emitted) and ($i==0 or $s.cursor==$a.steps[$i-1].source); "bad native transition: \($g.id)/\($name)/\($i)")) |
  reduce ($a.interventions[]? | select(.discarded|not) | select(.applied)) as $event (. ;
    $a.steps[$event.step] as $s |
    require($s.cursor==$event.from and $s.source==$event.to and $s.emitted==$p.prompt[$event.to] and ($event.step+1 >= ($a.steps|length) or $a.steps[$event.step+1].cursor==$event.to); "bad intervention: \($g.id)/\($name)")) |
  {first:$first, exact:($first==null), eos:($a.reply[-1]==1), retained:[$a.interventions[]? | select(.discarded|not)], discarded:[$a.interventions[]? | select(.discarded)], events:[$a.interventions[]? | select(.discarded|not) | select(.applied) | . + {token:$p.prompt[.to],next_cursor:($a.steps[.step+1].cursor // null)}]};
$grades[0].rows as $rows | $predictions[0] as $pred |
require(([$rows[]|select(.reached)|.id]|sort)==([$pred[].id]|sort) and ([$pred[].id]|length)==([$pred[].id]|unique|length); "reached prediction set differs") |
[ $rows[] | select(.reached) as $g | ($pred[]|select(.id==$g.id)) as $p |
  checkarm($p;$g;"promoted") as $b | checkarm($p;$g;"current") as $c | checkarm($p;$g;"original") as $o |
  require($g.role!="home" or ($p.current==$p.promoted and $p.original==$p.promoted); "home control differs: \($g.id)") |
  {id:$g.id,role:$g.role,cohort:$g.cohort, baseline:$b,current:$c,original:$o,
   current_movement:move($b.first;$c.first),original_movement:move($b.first;$o.first),
   original_vs_current:move($c.first;$o.first),
   same_token_different_source: (($c.events|length)>0 and ($o.events|length)>0 and $c.events[0].token==$o.events[0].token and $c.events[0].to!=$o.events[0].to)} ] as $out |
{attempted:($rows|length), reached:($out|length), rows:$out}
