#!/bin/bash
echo "=== CONTINUOUS RESEARCH MONITOR ==="
echo "Started: $(date)"
echo ""

last_count=0
while true; do
  if [ -f research_results.json ]; then
    count=$(jq 'length' research_results.json 2>/dev/null || echo 0)
    tokens=$(jq '[.[] | select(.is_token == true)] | length' research_results.json 2>/dev/null || echo 0)
    odin=$(jq '[.[] | select(.project == "ODIN.fun")] | length' research_results.json 2>/dev/null || echo 0)
    
    if [ "$count" -ne "$last_count" ]; then
      pct=$((100 * count / 2433))
      echo "$(date +%H:%M:%S) - Progress: $count/2,433 ($pct%) | Tokens: $tokens (ODIN: $odin)"
      last_count=$count
    fi
    
    # Check if complete
    if [ "$count" -ge 2433 ]; then
      echo ""
      echo "================================"
      echo "RESEARCH COMPLETE!"
      echo "================================"
      echo "Running consolidation..."
      python3 consolidate_results.py
      echo ""
      echo "All files generated. Research finished at $(date)"
      break
    fi
  fi
  
  sleep 30
done
