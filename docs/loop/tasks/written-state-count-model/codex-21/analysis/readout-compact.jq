# Keep aggregate counts and paired changes. Full IDs, traces and texts stay in readout.json.
del(.rows)
| .role_pairs |= map(select(.changed_query or .changed_ids or .old_source != .new_source))
| .value_pairs |= map(select(.changed_query or .changed_ids or .old_source != .new_source or
                             .earlier_unseen_gold or (.bypassed_unseen|length)>0))
