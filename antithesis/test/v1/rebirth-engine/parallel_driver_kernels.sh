#!/usr/bin/env sh
# Every registered unit except the whole-engine game (its own driver), with
# platform-chosen inputs. The list comes from the registry, so new units join.
units=""
for unit in $(rebirth-dst --list | cut -d' ' -f1); do
  [ "$unit" = game ] || units="${units:+$units,}$unit"
done
exec rebirth-dst --entropy antithesis --units "$units" --cases 2
