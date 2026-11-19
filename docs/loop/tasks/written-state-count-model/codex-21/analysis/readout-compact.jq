# Keep aggregate counts and paired changes. Full IDs, traces and texts stay in readout.json.
del(.rows)
| .role_pairs |= map(select(.changed_query or .changed_ids or .old_source_offset != .new_source_offset))
| .value_pairs |= map(select(.changed_query or .changed_ids or .old_source_offset != .new_source_offset or
                             .earlier_unseen_gold or (.bypassed_unseen|length)>0))
